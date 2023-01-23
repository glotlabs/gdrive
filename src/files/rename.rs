use crate::common::delegate::UploadDelegateConfig;
use crate::common::hub_helper;
use crate::files;
use crate::files::update::PatchFile;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Clone, Debug)]
pub struct Config {
    pub file_id: String,
    pub name: String,
}

pub async fn rename(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;
    let delegate_config = UploadDelegateConfig::default();

    let old_file = files::info::get_file(&hub, &config.file_id)
        .await
        .map_err(Error::GetFile)?;

    println!(
        "Renaming {} to {}",
        old_file.name.unwrap_or_default(),
        config.name
    );

    let patch_file = PatchFile::new(config.file_id).with_name(&config.name);

    files::update::update_metadata(&hub, delegate_config, patch_file)
        .await
        .map_err(Error::Rename)?;

    Ok(())
}

#[derive(Debug)]
pub enum Error {
    Hub(hub_helper::Error),
    GetFile(google_drive3::Error),
    Rename(google_drive3::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Hub(err) => write!(f, "{}", err),
            Error::GetFile(err) => {
                write!(f, "Failed to get file: {}", err)
            }
            Error::Rename(err) => {
                write!(f, "Failed to rename file: {}", err)
            }
        }
    }
}
