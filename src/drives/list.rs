use crate::common::delegate::UploadDelegate;
use crate::common::delegate::UploadDelegateConfig;
use crate::common::hub_helper;
use crate::common::table;
use crate::common::table::Table;
use crate::hub::Hub;
use std::error;
use std::fmt;
use std::io;

pub struct Config {
    pub skip_header: bool,
    pub field_separator: String,
}

pub async fn list(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;
    let delegate_config = UploadDelegateConfig::default();

    let drives = list_drives(&hub, delegate_config)
        .await
        .map_err(Error::ListDrives)?;

    print_drives_table(&config, drives);

    Ok(())
}

fn print_drives_table(config: &Config, drives: Vec<google_drive3::api::Drive>) {
    let mut values: Vec<[String; 2]> = vec![];

    for drive in drives {
        values.push([
            // fmt
            drive.id.unwrap_or_default(),
            drive.name.unwrap_or_default(),
        ])
    }

    let table = Table {
        header: ["Id", "Name"],
        values,
    };

    let _ = table::write(
        io::stdout(),
        table,
        &table::DisplayConfig {
            skip_header: config.skip_header,
            separator: config.field_separator.clone(),
        },
    );
}

pub async fn list_drives(
    hub: &Hub,
    delegate_config: UploadDelegateConfig,
) -> Result<Vec<google_drive3::api::Drive>, google_drive3::Error> {
    let mut delegate = UploadDelegate::new(delegate_config);

    let (_, drives_list) = hub
        .drives()
        .list()
        .add_scope(google_drive3::api::Scope::Full)
        .delegate(&mut delegate)
        .doit()
        .await?;

    Ok(drives_list.drives.unwrap_or_default())
}

#[derive(Debug)]
pub enum Error {
    Hub(hub_helper::Error),
    ListDrives(google_drive3::Error),
}

impl error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Hub(err) => write!(f, "{}", err),
            Error::ListDrives(err) => {
                write!(f, "Failed to list drives: {}", err)
            }
        }
    }
}
