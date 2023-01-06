use std::error;
use std::fmt::Display;
use std::fmt::Formatter;

use crate::common::drive_file;
use crate::common::hub_helper;
use crate::files;

pub struct Config {
    pub file_id: String,
    pub delete_directories: bool,
}

pub async fn delete(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;

    let file = files::info::get_file(&hub, &config.file_id)
        .await
        .map_err(Error::GetFile)?;

    err_if_directory(&file, &config)?;

    hub.files()
        .delete(&config.file_id)
        .supports_all_drives(true)
        .add_scope(google_drive3::api::Scope::Full)
        .doit()
        .await
        .map_err(Error::DeleteFile)?;

    println!("Deleted '{}'", file.name.unwrap_or_default());

    Ok(())
}

#[derive(Debug)]
pub enum Error {
    Hub(hub_helper::Error),
    GetFile(google_drive3::Error),
    DeleteFile(google_drive3::Error),
    IsDirectory(String),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Hub(err) => write!(f, "{}", err),
            Error::GetFile(err) => write!(f, "Failed getting file: {}", err),
            Error::DeleteFile(err) => write!(f, "Failed to delete file: {}", err),
            Error::IsDirectory(name) => write!(
                f,
                "'{}' is a directory, use --recursive to delete directories",
                name
            ),
        }
    }
}

fn err_if_directory(file: &google_drive3::api::File, config: &Config) -> Result<(), Error> {
    if drive_file::is_directory(file) && !config.delete_directories {
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
