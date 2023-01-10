use crate::common::chunk_size::ChunkSize;
use crate::common::delegate::Backoff;
use crate::common::delegate::BackoffConfig;
use crate::common::delegate::UploadDelegate;
use crate::common::delegate::UploadDelegateConfig;
use crate::common::file_info;
use crate::common::file_info::FileInfo;
use crate::common::hub_helper;
use crate::files;
use crate::files::info::DisplayConfig;
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
    pub parents: Option<Vec<String>>,
    pub chunk_size: ChunkSize,
}

pub async fn upload(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;

    let mut delegate = UploadDelegate::new(UploadDelegateConfig {
        chunk_size: config.chunk_size.in_bytes(),
        backoff: Backoff::new(BackoffConfig {
            max_retries: 100000,
            min_sleep: Duration::from_secs(1),
            max_sleep: Duration::from_secs(30),
        }),
    });

    let file = fs::File::open(&config.file_path)
        .map_err(|err| Error::OpenFile(config.file_path.clone(), err))?;

    let file_info = FileInfo::from_file(
        &file,
        &file_info::Config {
            file_path: config.file_path.clone(),
            mime_type: config.mime_type,
            parents: config.parents,
        },
    )
    .map_err(Error::FileInfo)?;

    let reader = std::io::BufReader::new(file);

    println!("Uploading {}", config.file_path.display());

    let file = upload_file(&hub, reader, file_info, &mut delegate)
        .await
        .map_err(Error::Upload)?;

    println!("File successfully uploaded");

    let fields = files::info::prepare_fields(&file, &DisplayConfig::default());
    files::info::print_fields(&fields);

    Ok(())
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
        parents: file_info.parents,
        ..google_drive3::api::File::default()
    };

    let req = hub
        .files()
        .create(dst_file)
        .param("fields", "id,name,size,createdTime,modifiedTime,md5Checksum,mimeType,parents,shared,description,webContentLink,webViewLink")
        .add_scope(google_drive3::api::Scope::Full)
        .delegate(delegate)
        .supports_all_drives(true);

    let (_, file) = if file_info.size > 0 {
        req.upload_resumable(src_file, file_info.mime_type).await?
    } else {
        req.upload(src_file, file_info.mime_type).await?
    };

    Ok(file)
}

#[derive(Debug)]
pub enum Error {
    Hub(hub_helper::Error),
    FileInfo(file_info::Error),
    OpenFile(PathBuf, io::Error),
    Upload(google_drive3::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Hub(err) => write!(f, "{}", err),
            Error::FileInfo(err) => write!(f, "{}", err),
            Error::OpenFile(path, err) => {
                write!(f, "Failed to open file '{}': {}", path.display(), err)
            }
            Error::Upload(err) => write!(f, "Failed to upload file: {}", err),
        }
    }
}
