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
    pub max: Option<i32>,
}

pub async fn list(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;
    let delegate_config = UploadDelegateConfig::default();

    let drives = list_drives(&hub, delegate_config, config.max)
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
    max: Option<i32>,
) -> Result<Vec<google_drive3::api::Drive>, google_drive3::Error> {
    let mut delegate = UploadDelegate::new(delegate_config);
    let mut all_drives = Vec::new();
    let mut page_token: Option<String> = None;
    
    // Default to 10 if the user does not provide a max
    let target_max = max.unwrap_or(10);
    // API limit is 100; keep page size within target_max and the API cap
    let page_size = target_max.min(100);

    loop {
        let mut request = hub
            .drives()
            .list()
            .page_size(page_size)
            .add_scope(google_drive3::api::Scope::Full);
        
        if let Some(token) = page_token {
            request = request.page_token(&token);
        }
        
        let (_, drives_list) = request
            .delegate(&mut delegate)
            .doit()
            .await?;

        if let Some(mut drives) = drives_list.drives {
            all_drives.append(&mut drives);
        }
        
        // Check if we've reached the target max or there are no more pages
        if all_drives.len() >= target_max as usize || drives_list.next_page_token.is_none() {
            break;
        }
        
        page_token = drives_list.next_page_token;
    }
    
    // Truncate to exact max if specified
    if let Some(max_count) = max {
        all_drives.truncate(max_count as usize);
    }

    Ok(all_drives)
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
