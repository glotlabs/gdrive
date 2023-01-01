use crate::common::hub_helper;
use crate::hub::Hub;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;

pub struct Config {
    pub file_id: String,
}

pub async fn info(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;

    let file = get_file(&hub, &config.file_id)
        .await
        .map_err(Error::GetFile)?;

    print_file_info(&file);

    Ok(())
}

pub async fn get_file(
    hub: &Hub,
    file_id: &str,
) -> Result<google_drive3::api::File, google_drive3::Error> {
    let (_, file) = hub
        .files()
        .get(file_id)
        .supports_all_drives(true)
        .add_scope(google_drive3::api::Scope::Full)
        .doit()
        .await?;

    Ok(file)
}

fn print_file_info(file: &google_drive3::api::File) {
    println!("Id: {}", format_optional_field(&file.id));
    println!("Name: {}", format_optional_field(&file.name));
    println!("Mime: {}", format_optional_field(&file.mime_type));

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
}

fn format_optional_field(field: &Option<String>) -> String {
    match field {
        Some(s) => s.to_string(),
        None => String::from("N/A"),
    }
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
