use crate::common::delegate::Backoff;
use crate::common::delegate::BackoffConfig;
use crate::common::delegate::UploadDelegate;
use crate::common::delegate::UploadDelegateConfig;
use crate::common::hub_helper;
use crate::hub::Hub;
use mime::Mime;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

pub struct Config {
    pub file_path: PathBuf,
    pub mime_type: Option<Mime>,
}

pub async fn upload(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;

    let mut delegate = UploadDelegate::new(UploadDelegateConfig {
        chunk_size: 1 << 23,
        backoff: Backoff::new(BackoffConfig {
            max_retries: 20,
            min_sleep: Duration::from_secs(1),
            max_sleep: Duration::from_secs(30),
        }),
    });

    let mime_type = config.mime_type.unwrap_or_else(|| {
        mime_guess::from_path(&config.file_path)
            .first()
            .unwrap_or(mime::APPLICATION_OCTET_STREAM)
    });

    let file_name = config
        .file_path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .ok_or(Error::InvalidFilePath(config.file_path.clone()))?;

    let file_info = FileInfo {
        name: file_name,
        mime_type,
    };

    let file = fs::File::open(&config.file_path)
        .map_err(|err| Error::OpenFile(config.file_path.clone(), err))?;
    let reader = std::io::BufReader::new(file);

    println!("Uploading {}", config.file_path.display());

    let file = upload_file(&hub, reader, file_info, &mut delegate)
        .await
        .map_err(Error::Upload)?;

    println!(
        "File successfully uploaded with id: {}",
        file.id.unwrap_or_default()
    );

    Ok(())
}

pub struct FileInfo {
    pub name: String,
    pub mime_type: mime::Mime,
}

pub async fn upload_file<'a, RS>(
    hub: &Hub,
    src_file: RS,
    file_info: FileInfo,
    delegate: &'a mut dyn google_drive3::client::Delegate,
) -> Result<google_drive3::api::File, google_drive3::Error>
where
    RS: google_drive3::client::ReadSeek,
{
    let dst_file = google_drive3::api::File {
        name: Some(file_info.name),
        ..google_drive3::api::File::default()
    };

    let (_body, file) = hub
        .files()
        .create(dst_file)
        .add_scope(google_drive3::api::Scope::Full)
        .delegate(delegate)
        .supports_all_drives(true)
        .upload_resumable(src_file, file_info.mime_type)
        .await?;

    Ok(file)
}

#[derive(Debug)]
pub enum Error {
    Hub(hub_helper::Error),
    InvalidFilePath(PathBuf),
    OpenFile(PathBuf, io::Error),
    Upload(google_drive3::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Hub(err) => write!(f, "{}", err),
            Error::InvalidFilePath(path) => write!(f, "Invalid file path: {}", path.display()),
            Error::OpenFile(path, err) => {
                write!(f, "Failed to open file '{}': {}", path.display(), err)
            }
            Error::Upload(err) => write!(f, "Failed to upload file: {}", err),
        }
    }
}
