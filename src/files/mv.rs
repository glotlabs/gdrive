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

pub async fn mv(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;
    let delegate_config = UploadDelegateConfig::default();

    let old_file = files::info::get_file(&hub, &config.file_id)
        .await
        .map_err(Error::GetFile)?;

    let old_parent_id = get_old_parent_id(&old_file)?;

    let old_parent = files::info::get_file(&hub, &old_parent_id)
        .await
        .map_err(|err| Error::GetOldParent(old_parent_id.clone(), err))?;

    let new_parent = files::info::get_file(&hub, &config.to_folder_id)
        .await
        .map_err(Error::GetNewParent)?;

    err_if_not_directory(&new_parent)?;

    println!(
        "Moving '{}' from '{}' to '{}'",
        old_file.name.unwrap_or_default(),
        old_parent.name.unwrap_or_default(),
        new_parent.name.unwrap_or_default()
    );

    let change_parent_config = ChangeParentConfig {
        file_id: config.file_id,
        old_parent_id,
        new_parent_id: config.to_folder_id,
    };

    change_parent(&hub, delegate_config, &change_parent_config)
        .await
        .map_err(Error::Move)?;

    Ok(())
}

pub struct ChangeParentConfig {
    pub file_id: String,
    pub old_parent_id: String,
    pub new_parent_id: String,
}

pub async fn change_parent(
    hub: &Hub,
    delegate_config: UploadDelegateConfig,
    config: &ChangeParentConfig,
) -> Result<google_drive3::api::File, google_drive3::Error> {
    let mut delegate = UploadDelegate::new(delegate_config);

    let empty_file = google_drive3::api::File::default();

    let (_, file) = hub
        .files()
        .update(empty_file, &config.file_id)
        .remove_parents(&config.old_parent_id)
        .add_parents(&config.new_parent_id)
        .param("fields", "id,name,size,createdTime,modifiedTime,md5Checksum,mimeType,parents,shared,description,webContentLink,webViewLink")
        .add_scope(google_drive3::api::Scope::Full)
        .delegate(&mut delegate)
        .supports_all_drives(true)
        .doit_without_upload().await?;

    Ok(file)
}

#[derive(Debug)]
pub enum Error {
    Hub(hub_helper::Error),
    GetFile(google_drive3::Error),
    GetOldParent(String, google_drive3::Error),
    GetNewParent(google_drive3::Error),
    NoParents,
    MultipleParents,
    NotADirectory,
    Move(google_drive3::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Error::Hub(err) => write!(f, "{}", err),
            Error::GetFile(err) => {
                write!(f, "Failed to get file: {}", err)
            }
            Error::GetNewParent(err) => {
                write!(f, "Failed to get new parent: {}", err)
            }
            Error::GetOldParent(id, err) => {
                write!(f, "Failed to get old parent '{}': {}", id, err)
            }
            Error::NoParents => {
                write!(f, "File has no parents")
            }
            Error::MultipleParents => {
                write!(f, "Can't move file with multiple parents")
            }
            Error::NotADirectory => {
                write!(f, "New parent is not a directory")
            }
            Error::Move(err) => {
                write!(f, "Failed to move file: {}", err)
            }
        }
    }
}

fn get_old_parent_id(file: &google_drive3::api::File) -> Result<String, Error> {
    match &file.parents {
        None => {
            // fmt
            Err(Error::NoParents)
        }

        Some(parents) => match &parents[..] {
            [] => Err(Error::NoParents),
            [parent_id] => Ok(parent_id.to_string()),
            _ => Err(Error::MultipleParents),
        },
    }
}

fn err_if_not_directory(file: &google_drive3::api::File) -> Result<(), Error> {
    if !drive_file::is_directory(file) {
        Err(Error::NotADirectory)
    } else {
        Ok(())
    }
}
