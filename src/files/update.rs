use crate::common::chunk_size::ChunkSize;
use crate::common::delegate::BackoffConfig;
use crate::common::delegate::UploadDelegate;
use crate::common::delegate::UploadDelegateConfig;
use crate::common::file_info;
use crate::common::file_info::FileInfo;
use crate::common::hub_helper;
use crate::files;
use crate::files::info;
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
    pub file_id: String,
    pub file_path: PathBuf,
    pub mime_type: Option<Mime>,
    pub chunk_size: ChunkSize,
    pub print_chunk_errors: bool,
    pub print_chunk_info: bool,
}

pub async fn update(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;

    let delegate_config = UploadDelegateConfig {
        chunk_size: config.chunk_size.in_bytes(),
        backoff_config: BackoffConfig {
            max_retries: 20,
            min_sleep: Duration::from_secs(1),
            max_sleep: Duration::from_secs(60),
        },
        print_chunk_errors: config.print_chunk_errors,
        print_chunk_info: config.print_chunk_info,
    };

    let file = fs::File::open(&config.file_path)
        .map_err(|err| Error::OpenFile(config.file_path.clone(), err))?;

    let drive_file = info::get_file(&hub, &config.file_id)
        .await
        .map_err(Error::GetFile)?;

    let file_info = FileInfo::from_file(
        &file,
        &file_info::Config {
            file_path: config.file_path.clone(),
            mime_type: config.mime_type,
            parents: drive_file.parents.clone(),
        },
    )
    .map_err(Error::FileInfo)?;

    let reader = std::io::BufReader::new(file);

    println!(
        "Updating {} with {}",
        config.file_id,
        config.file_path.display()
    );

    let file = update_file(&hub, reader, &config.file_id, file_info, delegate_config)
        .await
        .map_err(Error::Update)?;

    println!("File successfully updated");

    let fields = files::info::prepare_fields(&file, &DisplayConfig::default());
    files::info::print_fields(&fields);

    Ok(())
}

pub async fn update_file<'a, RS>(
    hub: &Hub,
    src_file: RS,
    file_id: &str,
    file_info: FileInfo,
    delegate_config: UploadDelegateConfig,
) -> Result<google_drive3::api::File, google_drive3::Error>
where
    RS: google_drive3::client::ReadSeek,
{
    let dst_file = google_drive3::api::File {
        name: Some(file_info.name),
        ..google_drive3::api::File::default()
    };

    let mut delegate = UploadDelegate::new(delegate_config);

    let req = hub
        .files()
        .update(dst_file, &file_id)
        .param("fields", "id,name,size,createdTime,modifiedTime,md5Checksum,mimeType,parents,shared,description,webContentLink,webViewLink")
        .add_scope(google_drive3::api::Scope::Full)
        .delegate(&mut delegate)
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
    GetFile(google_drive3::Error),
    Update(google_drive3::Error),
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
            Error::GetFile(err) => write!(f, "Failed to get file: {}", err),
            Error::Update(err) => write!(f, "Failed to update file: {}", err),
        }
    }
}
