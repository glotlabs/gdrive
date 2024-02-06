use crate::common::drive_file;
use crate::common::file_tree_drive;
use crate::common::file_tree_drive::FileTreeDrive;
use crate::common::hub_helper;

use crate::files;
use crate::files::list;
use crate::files::list::ListQuery;
use crate::files::FileExtension;
use crate::hub::Hub;

use futures::stream;
use futures::stream::StreamExt;

use futures::TryStreamExt;
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
use std::path::Path;
use std::path::PathBuf;

use tokio_util::compat::FuturesAsyncReadCompatExt;
use tokio_util::io::InspectReader;

use super::list::list_files;

type GFile = google_drive3::api::File;

pub struct Config {
    pub file_id: String,
    pub existing_file_action: ExistingFileAction,
    pub follow_shortcuts: bool,
    pub download_directories: bool,
    pub parallelisme: usize,
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

pub async fn _download_file(
    hub: &Hub,
    file_path: impl AsRef<Path>,
    file: &GFile,
) -> Result<(), Error> {
    let file_id = file.id.as_ref().ok_or_else(|| Error::MissingFileName)?;
    let body = download_file(&hub, file_id.as_str())
        .await
        .map_err(Error::DownloadFile)?;

    let file_path = file_path.as_ref();

    println!("Downloading file '{}'", file_path.display());
    save_body_to_file(body, &file_path, None).await?;

    Ok(())
}

pub async fn _download_dir(hub: &Hub, file: GFile, config: &Config) -> Result<(), Error> {
    let root_path = config.canonical_destination_root()?;
    let file_name = file.name.as_ref().ok_or_else(|| Error::MissingFileName)?;
    let path = root_path.join(file_name.as_str());

    stream::unfold(vec![(path, file)], |mut to_visit| async {
        let (path, file) = to_visit.pop()?;
        let file_id = file.id.as_ref()?;
        let files = list_files(
            &hub,
            &list::ListFilesConfig {
                query: ListQuery::FilesInFolder {
                    folder_id: file_id.clone(),
                },
                order_by: Default::default(),
                max_files: usize::MAX,
            },
        )
        .await;

        let file_stream = match files {
            Ok(files) => {
                let (dirs, others): (Vec<_>, Vec<_>) =
                    files.into_iter().partition(|f| f.is_directory()); // TODO: drain filter
                to_visit.extend(
                    dirs.into_iter()
                        .filter_map(|file| Some((path.join(file.name.as_ref()?), file))),
                );
                stream::iter(
                    others
                        .into_iter()
                        .filter_map(move |file| Some((path.join(file.name.as_ref()?), file))),
                )
                .map(Ok)
                .left_stream()
            }
            Err(err) => stream::once(async {
                Err(Error::CreateFileTree(file_tree_drive::Error::ListFiles(
                    err,
                )))
            })
            .right_stream(),
        };

        Some((file_stream, to_visit))
    })
    .flatten()
    .map(|file| async move {
        match file {
            Ok((path, file)) => _download_file(&hub, &path, &file).await,
            Err(_err) => Err(Error::MissingFileName), // TODO: fix error
        }
    })
    .buffer_unordered(config.parallelisme)
    .collect::<Vec<_>>()
    .await;

    Ok(())
}

pub async fn download(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;

    let file = files::info::get_file(&hub, &config.file_id)
        .await
        .map_err(Error::GetFile)?;

    err_if_file_exists(&file, &config)?;
    err_if_directory(&file, &config)?;
    err_if_shortcut(&file, &config)?;

    if drive_file::is_shortcut(&file) {
        // let target_file_id = file.shortcut_details.and_then(|details| details.target_id);

        // err_if_shortcut_target_is_missing(&target_file_id)?;

        // download(Config {
        //     file_id: target_file_id.unwrap_or_default(),
        //     ..config
        // })
        // .await?;
    } else if drive_file::is_directory(&file) {
        _download_dir(&hub, file, &config).await?;
    } else {
        // download_regular(&hub, &file, &config).await?;
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
    let tree = FileTreeDrive::from_file(&hub, &file)
        .await
        .map_err(Error::CreateFileTree)?;

    let tree_info = tree.info();

    println!(
        "Found {} files in {} directories with a total size of {}",
        tree_info.file_count,
        tree_info.folder_count,
        human_bytes(tree_info.total_file_size as f64)
    );

    let root_path = config.canonical_destination_root()?;

    for folder in &tree.folders() {
        let folder_path = folder.relative_path();
        let abs_folder_path = root_path.join(&folder_path);

        println!("Creating directory {}", folder_path.display());
        fs::create_dir_all(&abs_folder_path)
            .map_err(|err| Error::CreateDirectory(abs_folder_path, err))?;

        for file in folder.files() {
            let file_path = file.relative_path();
            let abs_file_path = root_path.join(&file_path);

            if local_file_is_identical(&abs_file_path, &file) {
                continue;
            }

            let body = download_file(&hub, &file.drive_id)
                .await
                .map_err(Error::DownloadFile)?;

            println!("Downloading file '{}'", file_path.display());
            save_body_to_file(body, &abs_file_path, file.md5.clone()).await?;
        }
    }

    println!(
        "Downloaded {} files in {} directories with a total size of {}",
        tree_info.file_count,
        tree_info.folder_count,
        human_bytes(tree_info.total_file_size as f64)
    );

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
    CreateFileTree(file_tree_drive::Error),
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
            Error::CreateFileTree(err) => write!(f, "Failed to create file tree: {}", err),
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
    body: hyper::Body,
    file_path: impl AsRef<Path>,
    expected_md5: Option<String>,
) -> Result<(), Error> {
    let file_path = file_path.as_ref();
    // Create temporary file

    tokio::fs::create_dir_all(file_path.parent().unwrap())
        .await
        .map_err(|err| Error::CreateDirectory(file_path.to_path_buf(), err))?;

    let tmp_file_path = file_path.with_extension("incomplete");
    let mut file = tokio::fs::File::create(&tmp_file_path)
        .await
        .map_err(Error::CreateFile)?;

    let mut md5 = md5::Context::new();

    let body = body
        .into_stream()
        .map(|result| {
            result.map_err(|_error| std::io::Error::new(std::io::ErrorKind::Other, "Error!"))
        })
        .into_async_read()
        .compat();

    let mut body = InspectReader::new(body, |bytes| md5.consume(&bytes));

    tokio::io::copy(&mut body, &mut file)
        .await
        .map_err(|err| Error::WriteChunk(err))?;

    // Check md5
    err_if_md5_mismatch(expected_md5, format!("{:x}", md5.compute()))?;

    // Rename temporary file to final file
    tokio::fs::rename(&tmp_file_path, &file_path)
        .await
        .map_err(Error::RenameFile)
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

fn local_file_is_identical(path: &PathBuf, file: &file_tree_drive::File) -> bool {
    if path.exists() {
        let file_md5 = compute_md5_from_path(path).unwrap_or_else(|err| {
            eprintln!(
                "Warning: Error while computing md5 of '{}': {}",
                path.display(),
                err
            );

            String::new()
        });

        file.md5.clone().map(|md5| md5 == file_md5).unwrap_or(false)
    } else {
        false
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
