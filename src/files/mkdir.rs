use crate::common::chunk_size::ChunkSize;
use crate::common::delegate::BackoffConfig;
use crate::common::delegate::UploadDelegate;
use crate::common::delegate::UploadDelegateConfig;
use crate::common::drive_file::MIME_TYPE_FOLDER;
use crate::common::empty_file::EmptyFile;
use crate::common::hub_helper;
use crate::hub::Hub;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Config {
    pub id: Option<String>,
    pub name: String,
    pub parents: Option<Vec<String>>,
    pub print_only_id: bool,
}

pub async fn mkdir(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;

    let delegate_config = UploadDelegateConfig {
        chunk_size: ChunkSize::default().in_bytes(),
        backoff_config: BackoffConfig {
            max_retries: 100,
            min_sleep: Duration::from_secs(1),
            max_sleep: Duration::from_secs(30),
        },
        print_chunk_errors: false,
        print_chunk_info: false,
    };

    let file = create_directory(&hub, &config, delegate_config)
        .await
        .map_err(Error::CreateDirectory)?;

    if config.print_only_id {
        print!("{}", file.id.unwrap_or_default())
    } else {
        println!(
            "Created directory '{}' with id: {}",
            config.name,
            file.id.unwrap_or_default()
        );
    }

    Ok(())
}

pub async fn create_directory(
    hub: &Hub,
    config: &Config,
    delegate_config: UploadDelegateConfig,
) -> Result<google_drive3::api::File, google_drive3::Error> {
    let dst_file = google_drive3::api::File {
        id: config.id.clone(),
        name: Some(config.name.clone()),
        parents: config.parents.clone(),
        mime_type: Some(MIME_TYPE_FOLDER.to_string()),
        ..google_drive3::api::File::default()
    };

    let mut delegate = UploadDelegate::new(delegate_config);

    let req = hub
        .files()
        .create(dst_file)
        .param("fields", "id,name,size,createdTime,modifiedTime,md5Checksum,mimeType,parents,shared,description,webContentLink,webViewLink")
        .add_scope(google_drive3::api::Scope::Full)
        .delegate(&mut delegate)
        .supports_all_drives(true);

    let empty_file = EmptyFile();
    let mime_type: mime::Mime = MIME_TYPE_FOLDER.parse().unwrap();

    let (_, file) = req.upload(empty_file, mime_type).await?;

    Ok(file)
}

#[derive(Debug)]
pub enum Error {
    Hub(hub_helper::Error),
    CreateDirectory(google_drive3::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Hub(err) => write!(f, "{}", err),
            Error::CreateDirectory(err) => {
                write!(f, "Failed to create directory on drive: {}", err)
            }
        }
    }
}
