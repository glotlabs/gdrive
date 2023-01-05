use google_drive3::chrono;
use google_drive3::chrono::DateTime;
use human_bytes::human_bytes;

use crate::common::hub_helper;
use crate::hub::Hub;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;

pub struct Config {
    pub file_id: String,
    pub size_in_bytes: bool,
}

pub async fn info(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;

    let file = get_file(&hub, &config.file_id)
        .await
        .map_err(Error::GetFile)?;

    let fields = prepare_fields(
        &file,
        &DisplayConfig {
            size_in_bytes: config.size_in_bytes,
        },
    );

    print_fields(&fields);

    Ok(())
}

pub async fn get_file(
    hub: &Hub,
    file_id: &str,
) -> Result<google_drive3::api::File, google_drive3::Error> {
    let (_, file) = hub
        .files()
        .get(file_id)
        .param("fields", "id,name,size,createdTime,modifiedTime,md5Checksum,mimeType,parents,shared,description,webContentLink,webViewLink")
        .supports_all_drives(true)
        .add_scope(google_drive3::api::Scope::Full)
        .doit()
        .await?;

    Ok(file)
}

pub fn print_fields(fields: &Vec<Field>) {
    for field in fields {
        if let Some(value) = &field.value {
            println!("{}: {}", field.name, value);
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DisplayConfig {
    pub size_in_bytes: bool,
}

pub struct Field {
    pub name: String,
    pub value: Option<String>,
}

pub fn prepare_fields(file: &google_drive3::api::File, config: &DisplayConfig) -> Vec<Field> {
    // Id: 0B3X9GlR6EmbnNTk0SkV0bm5Hd0E
    // Name: gdrive-osx-x64
    // Path: gdrive-bin/gdrive-osx-x64
    // Mime: application/octet-stream
    // Size: 8.3 MB
    // Created: 2016-02-21 20:47:04
    // Modified: 2016-02-21 20:47:04
    // Md5sum: b607f29231a3b2d16098c4212516470f
    // Shared: True
    // Parents: 0B3X9GlR6EmbnY1RLVTk5VUtOVkk
    // ViewUrl: https://drive.google.com/file/d/0B3X9GlR6EmbnNTk0SkV0bm5Hd0E/view?usp=drivesdk
    // DownloadUrl: https://docs.google.com/uc?id=0B3X9GlR6EmbnNTk0SkV0bm5Hd0E&export=download

    // TODO: Path
    // TODO: DownloadUrl

    vec![
        Field {
            name: String::from("Id"),
            value: file.id.clone(),
        },
        Field {
            name: String::from("Name"),
            value: file.name.clone(),
        },
        Field {
            name: String::from("Mime"),
            value: file.mime_type.clone(),
        },
        Field {
            name: String::from("Size"),
            value: file.size.map(|bytes| format_bytes(bytes, config)),
        },
        Field {
            name: String::from("Created"),
            value: file.created_time.map(format_date_time),
        },
        Field {
            name: String::from("Modified"),
            value: file.modified_time.map(format_date_time),
        },
        Field {
            name: String::from("MD5"),
            value: file.md5_checksum.clone(),
        },
        Field {
            name: String::from("Shared"),
            value: file.shared.map(format_bool),
        },
        Field {
            name: String::from("Parents"),
            value: file.parents.as_ref().map(format_list),
        },
        Field {
            name: String::from("ViewUrl"),
            value: file.web_view_link.clone(),
        },
    ]
}

fn format_bool(b: bool) -> String {
    if b {
        String::from("True")
    } else {
        String::from("False")
    }
}

fn format_list(list: &Vec<String>) -> String {
    list.join(", ")
}

fn format_bytes(bytes: i64, config: &DisplayConfig) -> String {
    if config.size_in_bytes {
        bytes.to_string()
    } else {
        human_bytes(bytes as f64)
    }
}

fn format_date_time(utc_time: DateTime<chrono::Utc>) -> String {
    let local_time: DateTime<chrono::Local> = DateTime::from(utc_time);
    local_time.format("%Y-%m-%d %H:%M:%S").to_string()
}

#[derive(Debug)]
pub enum Error {
    Hub(hub_helper::Error),
    GetFile(google_drive3::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Hub(err) => write!(f, "{}", err),
            Error::GetFile(err) => write!(f, "Failed getting file: {}", err),
        }
    }
}
