use crate::common::drive_file;
use crate::common::file_tree_drive;
use crate::common::file_tree_drive::FileTreeDrive;
use crate::common::hub_helper;
use crate::common::md5_writer::Md5Writer;
use crate::files;
use crate::hub::Hub;
use futures::stream::StreamExt;
use google_drive3::hyper;
use human_bytes::human_bytes;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::PathBuf;

pub struct Config {
    pub file_id: String,
    pub existing_file_action: ExistingFileAction,
    pub download_directories: bool,
    pub destination_root: Option<PathBuf>,
}

impl Config {
    fn canonical_destination_root(&self) -> Result<PathBuf, Error> {
        if let Some(path) = &self.destination_root {
            if !path.exists() {
                Err(Error::DestinationPathDoesNotExist(path.clone()))
            } else if !path.is_dir() {
                Err(Error::DestinationPathNotADirectory(path.clone()))
            } else {
                path.canonicalize()
                    .map_err(|err| Error::CanonicalizeDestinationPath(path.clone(), err))
            }
        } else {
            let current_path = PathBuf::from(".");
            let canonical_current_path = current_path
                .canonicalize()
                .map_err(|err| Error::CanonicalizeDestinationPath(current_path.clone(), err))?;
            Ok(canonical_current_path)
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ExistingFileAction {
    Abort,
    Overwrite,
}

pub async fn download(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;

    let file = files::info::get_file(&hub, &config.file_id)
        .await
        .map_err(Error::GetFile)?;

    err_if_file_exists(&file, &config)?;
    err_if_directory(&file, &config)?;

    if drive_file::is_directory(&file) {
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
    let file_name = file.name.clone().ok_or(Error::MissingFileName)?;
    let root_path = config.canonical_destination_root()?;
    let abs_file_path = root_path.join(&file_name);

    let body = download_file(&hub, &config.file_id)
        .await
        .map_err(Error::DownloadFile)?;

    println!("Downloading {}", file_name);
    save_body_to_file(body, &abs_file_path, file.md5_checksum.clone()).await?;
    println!("Successfully downloaded {}", file_name);

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
            let body = download_file(&hub, &file.drive_id)
                .await
                .map_err(Error::DownloadFile)?;

            let file_path = file.relative_path();
            let abs_file_path = root_path.join(&file_path);

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

fn err_if_file_exists(file: &google_drive3::api::File, config: &Config) -> Result<(), Error> {
    let file_name = file.name.clone().ok_or(Error::MissingFileName)?;
    let root_path = config.canonical_destination_root()?;
    let file_path = root_path.join(&file_name);

    if file_path.exists() && config.existing_file_action == ExistingFileAction::Abort {
        Err(Error::FileExists(file_path.clone()))
    } else {
        Ok(())
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
