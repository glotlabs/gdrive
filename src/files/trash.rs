use crate::common::{drive_file, hub_helper};
use crate::files::info;
use crate::hub::Hub;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;

pub struct Config {
    pub file_id: String,
    pub trash_directories: bool,
}

pub async fn trash(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;

    let exists = info::get_file(&hub, &config.file_id)
        .await
        .map_err(Error::GetFile)?;

    err_if_directory(&exists, &config)?;
    
    if exists.trashed.is_some_and(|trashed| trashed == true) {
        println!("File is already trashed, exiting");
        return Ok(());
    }

    println!("Trashing {}", config.file_id);

    trash_file(&hub, &config.file_id)
        .await
        .map_err(Error::Update)?;

    println!("File successfully updated");

    Ok(())
}

pub async fn trash_file(hub: &Hub, file_id: &str) -> Result<(), google_drive3::Error> {
    let dst_file = google_drive3::api::File {
        trashed: Some(true),
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
    IsDirectory(String),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Hub(err) => write!(f, "{}", err),
            Error::GetFile(err) => write!(f, "Failed to get file: {}", err),
            Error::Update(err) => write!(f, "Failed to trash file: {}", err),
            Error::IsDirectory(name) => write!(
                f,
                "'{}' is a directory, use --recursive to trash directories",
                name
            ),
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

fn err_if_directory(file: &google_drive3::api::File, config: &Config) -> Result<(), Error> {
    if drive_file::is_directory(file) && !config.trash_directories {
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