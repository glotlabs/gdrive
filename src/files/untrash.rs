use crate::common::hub_helper;
use crate::files::info;
use crate::hub::Hub;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;

pub struct Config {
    pub file_id: String,
}

pub async fn untrash(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;

    let exists = info::get_file(&hub, &config.file_id)
        .await
        .map_err(Error::GetFile)?;

    if exists.trashed.is_some_and(|trashed| trashed == false) {
        println!("File is not trashed, exiting");
        return Ok(());
    }

    println!("Untrashing {}", config.file_id);

    untrash_file(&hub, &config.file_id)
        .await
        .map_err(Error::Update)?;

    println!("File successfully updated");

    Ok(())
}

pub async fn untrash_file(hub: &Hub, file_id: &str) -> Result<(), google_drive3::Error> {
    let dst_file = google_drive3::api::File {
        trashed: Some(false),
        ..google_drive3::api::File::default()
    };

    let req = hub
        .files()
        .update(dst_file, &file_id)
        .param("fields", "id,name,size,createdTime,modifiedTime,md5Checksum,mimeType,parents,shared,description,webContentLink,webViewLink")
        .add_scope(google_drive3::api::Scope::Full)
        .supports_all_drives(true);

    req.doit_without_upload().await?;

    Ok(())
}

#[derive(Debug)]
pub enum Error {
    Hub(hub_helper::Error),
    GetFile(google_drive3::Error),
    Update(google_drive3::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Hub(err) => write!(f, "{}", err),
            Error::GetFile(err) => write!(f, "Failed to get file: {}", err),
            Error::Update(err) => write!(f, "Failed to update file: {}", err),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PatchFile {
    id: String,
    file: google_drive3::api::File,
}

impl PatchFile {
    pub fn new(id: String) -> Self {
        Self {
            id,
            file: google_drive3::api::File::default(),
        }
    }

    pub fn with_name(&self, name: &str) -> Self {
        Self {
            file: google_drive3::api::File {
                name: Some(name.to_string()),
                ..self.file.clone()
            },
            ..self.clone()
        }
    }

    pub fn id(&self) -> String {
        self.id.clone()
    }

    pub fn file(&self) -> google_drive3::api::File {
        self.file.clone()
    }
}
