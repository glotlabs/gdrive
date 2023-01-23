use crate::common::delegate::UploadDelegate;
use crate::common::delegate::UploadDelegateConfig;
use crate::common::drive_file;
use crate::common::hub_helper;
use crate::files;
use crate::hub::Hub;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Clone, Debug)]
pub struct Config {
    pub file_id: String,
    pub to_folder_id: String,
}

pub async fn copy(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;
    let delegate_config = UploadDelegateConfig::default();

    let file = files::info::get_file(&hub, &config.file_id)
        .await
        .map_err(Error::GetFile)?;

    err_if_directory(&file)?;

    let to_parent = files::info::get_file(&hub, &config.to_folder_id)
        .await
        .map_err(Error::GetDestinationFolder)?;

    err_if_not_directory(&to_parent)?;

    println!(
        "Copying '{}' to '{}'",
        file.name.unwrap_or_default(),
        to_parent.name.unwrap_or_default()
    );

    let copy_config = CopyConfig {
        file_id: config.file_id,
        to_folder_id: config.to_folder_id,
    };

    copy_file(&hub, delegate_config, &copy_config)
        .await
        .map_err(Error::Copy)?;

    Ok(())
}

pub struct CopyConfig {
    pub file_id: String,
    pub to_folder_id: String,
}

pub async fn copy_file(
    hub: &Hub,
    delegate_config: UploadDelegateConfig,
    config: &CopyConfig,
) -> Result<google_drive3::api::File, google_drive3::Error> {
    let mut delegate = UploadDelegate::new(delegate_config);

    let file = google_drive3::api::File {
        parents: Some(vec![config.to_folder_id.clone()]),
        ..google_drive3::api::File::default()
    };

    let (_, file) = hub
        .files()
        .copy(file, &config.file_id)
        .param("fields", "id,name,size,createdTime,modifiedTime,md5Checksum,mimeType,parents,shared,description,webContentLink,webViewLink")
        .add_scope(google_drive3::api::Scope::Full)
        .delegate(&mut delegate)
        .supports_all_drives(true)
        .doit().await?;

    Ok(file)
}

#[derive(Debug)]
pub enum Error {
    Hub(hub_helper::Error),
    GetFile(google_drive3::Error),
    GetDestinationFolder(google_drive3::Error),
    DestinationNotADirectory,
    SourceIsADirectory,
    Copy(google_drive3::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Error::Hub(err) => write!(f, "{}", err),
            Error::GetFile(err) => {
                write!(f, "Failed to get file: {}", err)
            }
            Error::GetDestinationFolder(err) => {
                write!(f, "Failed to get destination folder: {}", err)
            }
            Error::DestinationNotADirectory => {
                write!(f, "Can only copy to a directory")
            }
            Error::SourceIsADirectory => {
                write!(f, "Copy directories is not supported")
            }
            Error::Copy(err) => {
                write!(f, "Failed to move file: {}", err)
            }
        }
    }
}

fn err_if_directory(file: &google_drive3::api::File) -> Result<(), Error> {
    if drive_file::is_directory(file) {
        Err(Error::SourceIsADirectory)
    } else {
        Ok(())
    }
}

fn err_if_not_directory(file: &google_drive3::api::File) -> Result<(), Error> {
    if !drive_file::is_directory(file) {
        Err(Error::DestinationNotADirectory)
    } else {
        Ok(())
    }
}
