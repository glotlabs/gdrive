use crate::common::drive_file;
use crate::common::drive_file::DocType;
use crate::common::hub_helper;
use crate::common::md5_writer::Md5Writer;
use crate::files;
use crate::hub::Hub;
use async_recursion::async_recursion;
use futures::stream::StreamExt;
use google_drive3::hyper;
use human_bytes::human_bytes;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

pub struct Config {
    pub file_id: String,
    pub existing_file_action: ExistingFileAction,
    pub follow_shortcuts: bool,
    pub download_directories: bool,
    pub destination: Destination,
}

impl Config {
    fn canonical_destination_root(&self) -> Result<PathBuf, Error> {
        match &self.destination {
            Destination::CurrentDir => {
                let current_path = PathBuf::from(".");
                let canonical_current_path = current_path
                    .canonicalize()
                    .map_err(|err| Error::CanonicalizeDestinationPath(current_path.clone(), err))?;
                Ok(canonical_current_path)
            }

            Destination::Path(path) => {
                if !path.exists() {
                    Err(Error::DestinationPathDoesNotExist(path.clone()))
                } else if !path.is_dir() {
                    Err(Error::DestinationPathNotADirectory(path.clone()))
                } else {
                    path.canonicalize()
                        .map_err(|err| Error::CanonicalizeDestinationPath(path.clone(), err))
                }
            }

            Destination::Stdout => {
                // fmt
                Err(Error::StdoutNotValidDestination)
            }
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Destination {
    CurrentDir,
    Path(PathBuf),
    Stdout,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ExistingFileAction {
    Abort,
    Overwrite,
}

#[async_recursion]
pub async fn download(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;

    let file = files::info::get_file(&hub, &config.file_id)
        .await
        .map_err(Error::GetFile)?;

    err_if_file_exists(&file, &config)?;
    err_if_directory(&file, &config)?;
    err_if_shortcut(&file, &config)?;

    if drive_file::is_shortcut(&file) {
        let target_file_id = file.shortcut_details.and_then(|details| details.target_id);

        err_if_shortcut_target_is_missing(&target_file_id)?;

        download(Config {
            file_id: target_file_id.unwrap_or_default(),
            ..config
        })
        .await?;
    } else if drive_file::is_directory(&file) {
        download_directory(&hub, &file, &config).await?;
    } else {
        download_regular(&hub, &file, &config).await?;
    }

    Ok(())
}

pub async fn download_regular(
    hub: &Hub,
    file: &google_drive3::api::File,
    config: &Config,
) -> Result<(), Error> {
    let body = download_file(&hub, &config.file_id)
        .await
        .map_err(Error::DownloadFile)?;

    match &config.destination {
        Destination::Stdout => {
            // fmt
            save_body_to_stdout(body).await?;
        }

        _ => {
            let file_name = file.name.clone().ok_or(Error::MissingFileName)?;
            let root_path = config.canonical_destination_root()?;
            let abs_file_path = root_path.join(&file_name);

            println!("Downloading {}", file_name);
            save_body_to_file(body, &abs_file_path, file.md5_checksum.clone()).await?;
            println!("Successfully downloaded {}", file_name);
        }
    }

    Ok(())
}

pub async fn download_directory(
    hub: &Hub,
    file: &google_drive3::api::File,
    config: &Config,
) -> Result<(), Error> {
    let root_path = config.canonical_destination_root()?;
    let dir_name = file.name.clone().ok_or(Error::MissingFileName)?;
    let dir_path = PathBuf::from(&dir_name);

    let mut stats = DownloadStats::default();
    download_directory_recursive(hub, file, &root_path, &dir_path, &mut stats).await?;

    println!(
        "Downloaded {} files in {} directories with a total size of {}",
        stats.file_count,
        stats.folder_count,
        human_bytes(stats.total_file_size as f64)
    );

    Ok(())
}

#[derive(Default)]
struct DownloadStats {
    file_count: u64,
    folder_count: u64,
    total_file_size: u64,
}

#[async_recursion]
async fn download_directory_recursive(
    hub: &Hub,
    dir_file: &google_drive3::api::File,
    root_path: &PathBuf,
    dir_path: &PathBuf,
    stats: &mut DownloadStats,
) -> Result<(), Error> {
    let abs_dir_path = root_path.join(dir_path);
    println!("Creating directory {}", dir_path.display());
    fs::create_dir_all(&abs_dir_path)
        .map_err(|err| Error::CreateDirectory(abs_dir_path.clone(), err))?;
    stats.folder_count += 1;

    let file_id = dir_file.id.clone().ok_or(Error::MissingFileName)?;
    let children = files::list::list_files(
        hub,
        &files::list::ListFilesConfig {
            query: files::list::ListQuery::FilesInFolder { folder_id: file_id },
            order_by: Default::default(),
            max_files: usize::MAX,
        },
    )
    .await
    .map_err(Error::ListFiles)?;

    for child in &children {
        if drive_file::is_directory(child) {
            let child_name = child.name.clone().ok_or(Error::MissingFileName)?;
            let child_path = dir_path.join(&child_name);
            download_directory_recursive(hub, child, root_path, &child_path, stats).await?;
        } else if drive_file::is_binary(child) {
            let file_name = child.name.clone().ok_or(Error::MissingFileName)?;
            let file_path = dir_path.join(&file_name);
            let abs_file_path = root_path.join(&file_path);

            if abs_file_path.exists() {
                let file_md5 = compute_md5_from_path(&abs_file_path).unwrap_or_default();
                if child.md5_checksum.as_deref() == Some(&file_md5) {
                    continue;
                }
            }

            let body = download_file(hub, child.id.as_deref().unwrap_or_default())
                .await
                .map_err(Error::DownloadFile)?;

            println!("Downloading file '{}'", file_path.display());
            save_body_to_file(body, &abs_file_path, child.md5_checksum.clone()).await?;
            stats.file_count += 1;
            stats.total_file_size += child.size.unwrap_or(0) as u64;
        } else if let Some(doc_type) = DocType::from_mime_type(
            child.mime_type.as_deref().unwrap_or_default(),
        ) {
            let file_name = child.name.clone().ok_or(Error::MissingFileName)?;
            let export_ext = doc_type.default_office_export_type();
            let export_name = format!("{}.{}", file_name, export_ext);
            let file_path = dir_path.join(&export_name);
            let abs_file_path = root_path.join(&file_path);

            let mime_type = export_ext.get_export_mime().unwrap();
            let body = files::export::export_file(hub, child.id.as_deref().unwrap_or_default(), &mime_type)
                .await
                .map_err(Error::ExportFile)?;

            println!("Exporting {} '{}'", doc_type, file_path.display());
            save_body_to_file(body, &abs_file_path, None).await?;
            stats.file_count += 1;
        }
    }

    Ok(())
}

pub async fn download_file(hub: &Hub, file_id: &str) -> Result<hyper::Body, google_drive3::Error> {
    let (response, _) = hub
        .files()
        .get(file_id)
        .supports_all_drives(true)
        .param("alt", "media")
        .add_scope(google_drive3::api::Scope::Full)
        .doit()
        .await?;

    Ok(response.into_body())
}

#[derive(Debug)]
pub enum Error {
    Hub(hub_helper::Error),
    GetFile(google_drive3::Error),
    DownloadFile(google_drive3::Error),
    ExportFile(google_drive3::Error),
    ListFiles(files::list::Error),
    MissingFileName,
    FileExists(PathBuf),
    IsDirectory(String),
    Md5Mismatch { expected: String, actual: String },
    CreateFile(io::Error),
    CreateDirectory(PathBuf, io::Error),
    CopyFile(io::Error),
    RenameFile(io::Error),
    ReadChunk(hyper::Error),
    WriteChunk(io::Error),
    DestinationPathDoesNotExist(PathBuf),
    DestinationPathNotADirectory(PathBuf),
    CanonicalizeDestinationPath(PathBuf, io::Error),
    MissingShortcutTarget,
    IsShortcut(String),
    StdoutNotValidDestination,
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Hub(err) => write!(f, "{}", err),
            Error::GetFile(err) => write!(f, "Failed getting file: {}", err),
            Error::DownloadFile(err) => write!(f, "Failed to download file: {}", err),
            Error::ExportFile(err) => write!(f, "Failed to export file: {}", err),
            Error::ListFiles(err) => write!(f, "Failed to list files: {}", err),
            Error::MissingFileName => write!(f, "File does not have a name"),
            Error::FileExists(path) => write!(
                f,
                "File '{}' already exists, use --overwrite to overwrite it",
                path.display()
            ),
            Error::IsDirectory(name) => write!(
                f,
                "'{}' is a directory, use --recursive to download directories",
                name
            ),
            Error::Md5Mismatch { expected, actual } => {
                // fmt
                write!(
                    f,
                    "MD5 mismatch, expected: {}, actual: {}",
                    expected, actual
                )
            }
            Error::CreateFile(err) => write!(f, "Failed to create file: {}", err),
            Error::CreateDirectory(path, err) => write!(
                f,
                "Failed to create directory '{}': {}",
                path.display(),
                err
            ),
            Error::CopyFile(err) => write!(f, "Failed to copy file: {}", err),
            Error::RenameFile(err) => write!(f, "Failed to rename file: {}", err),
            Error::ReadChunk(err) => write!(f, "Failed read from stream: {}", err),
            Error::WriteChunk(err) => write!(f, "Failed write to file: {}", err),
            Error::DestinationPathDoesNotExist(path) => {
                write!(f, "Destination path '{}' does not exist", path.display())
            }
            Error::DestinationPathNotADirectory(path) => {
                write!(
                    f,
                    "Destination path '{}' is not a directory",
                    path.display()
                )
            }
            Error::CanonicalizeDestinationPath(path, err) => write!(
                f,
                "Failed to canonicalize destination path '{}': {}",
                path.display(),
                err
            ),
            Error::MissingShortcutTarget => write!(f, "Shortcut does not have a target"),
            Error::IsShortcut(name) => write!(
                f,
                "'{}' is a shortcut, use --follow-shortcuts to download the file it points to",
                name
            ),
            Error::StdoutNotValidDestination => write!(
                f,
                "Stdout is not a valid destination for this combination of options"
            ),
        }
    }
}

// TODO: move to common
pub async fn save_body_to_file(
    mut body: hyper::Body,
    file_path: &PathBuf,
    expected_md5: Option<String>,
) -> Result<(), Error> {
    // Create temporary file
    let tmp_file_path = file_path.with_extension("incomplete");
    let file = File::create(&tmp_file_path).map_err(Error::CreateFile)?;

    // Wrap file in writer that calculates md5
    let mut writer = Md5Writer::new(file);

    // Read chunks from stream and write to file
    while let Some(chunk_result) = body.next().await {
        let chunk = chunk_result.map_err(Error::ReadChunk)?;
        writer.write_all(&chunk).map_err(Error::WriteChunk)?;
    }

    // Check md5
    err_if_md5_mismatch(expected_md5, writer.md5())?;

    // Rename temporary file to final file
    fs::rename(&tmp_file_path, &file_path).map_err(Error::RenameFile)
}

// TODO: move to common
pub async fn save_body_to_stdout(mut body: hyper::Body) -> Result<(), Error> {
    let mut stdout = io::stdout();

    // Read chunks from stream and write to stdout
    while let Some(chunk_result) = body.next().await {
        let chunk = chunk_result.map_err(Error::ReadChunk)?;
        stdout.write_all(&chunk).map_err(Error::WriteChunk)?;
    }

    Ok(())
}

fn err_if_file_exists(file: &google_drive3::api::File, config: &Config) -> Result<(), Error> {
    let file_name = file.name.clone().ok_or(Error::MissingFileName)?;

    let file_path = match &config.destination {
        Destination::CurrentDir => Some(PathBuf::from(".").join(file_name)),
        Destination::Path(path) => Some(path.join(file_name)),
        Destination::Stdout => None,
    };

    match file_path {
        Some(path) => {
            if path.exists() && config.existing_file_action == ExistingFileAction::Abort {
                Err(Error::FileExists(path.clone()))
            } else {
                Ok(())
            }
        }

        None => {
            // fmt
            Ok(())
        }
    }
}

fn err_if_directory(file: &google_drive3::api::File, config: &Config) -> Result<(), Error> {
    if drive_file::is_directory(file) && !config.download_directories {
        let name = file
            .name
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_default();
        Err(Error::IsDirectory(name))
    } else {
        Ok(())
    }
}

fn err_if_shortcut(file: &google_drive3::api::File, config: &Config) -> Result<(), Error> {
    if drive_file::is_shortcut(file) && !config.follow_shortcuts {
        let name = file
            .name
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_default();
        Err(Error::IsShortcut(name))
    } else {
        Ok(())
    }
}

fn err_if_shortcut_target_is_missing(target_id: &Option<String>) -> Result<(), Error> {
    if target_id.is_none() {
        Err(Error::MissingShortcutTarget)
    } else {
        Ok(())
    }
}

fn err_if_md5_mismatch(expected: Option<String>, actual: String) -> Result<(), Error> {
    let is_matching = expected.clone().map(|md5| md5 == actual).unwrap_or(true);

    if is_matching {
        Ok(())
    } else {
        Err(Error::Md5Mismatch {
            expected: expected.unwrap_or_default(),
            actual,
        })
    }
}

fn compute_md5_from_path(path: &PathBuf) -> Result<String, io::Error> {
    let input = File::open(path)?;
    let reader = BufReader::new(input);
    compute_md5_from_reader(reader)
}

fn compute_md5_from_reader<R: Read>(mut reader: R) -> Result<String, io::Error> {
    let mut context = md5::Context::new();
    let mut buffer = [0; 4096];

    loop {
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        context.consume(&buffer[..count]);
    }

    Ok(format!("{:x}", context.compute()))
}
