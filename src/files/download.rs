use crate::common::hub_helper;
use crate::files;
use crate::hub::Hub;
use google_drive3::hyper;
use google_drive3::hyper::body::Buf;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs::File;
use std::io;
use std::path::PathBuf;

pub struct Config {
    pub file_id: String,
}

pub async fn download(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;

    let file = files::info::get_file(&hub, &config.file_id)
        .await
        .map_err(Error::GetFile)?;

    let body = download_file(&hub, &config.file_id)
        .await
        .map_err(Error::DownloadFile)?;

    let file_name = file.name.ok_or(Error::MissingFileName)?;
    let file_path = PathBuf::from(&file_name);

    println!("Downloading {}", file_name);
    save_body_to_file(body, &file_path)
        .await
        .map_err(Error::SaveToFile)?;
    println!("Successfully downloaded {} ", file_name,);

    Ok(())
}

pub async fn download_file(hub: &Hub, file_id: &str) -> Result<hyper::Body, google_drive3::Error> {
    let (response, _) = hub
        .files()
        .get(file_id)
        .supports_all_drives(true)
        .param("alt", "media")
        .add_scope(google_drive3::api::Scope::Full)
        .doit()
        .await?;

    Ok(response.into_body())
}

#[derive(Debug)]
pub enum Error {
    Hub(hub_helper::Error),
    GetFile(google_drive3::Error),
    DownloadFile(google_drive3::Error),
    MissingFileName,
    SaveToFile(io::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Hub(err) => write!(f, "{}", err),
            Error::GetFile(err) => write!(f, "Failed getting file: {}", err),
            Error::DownloadFile(err) => write!(f, "Failed to download file: {}", err),
            Error::MissingFileName => write!(f, "File does not have a name"),
            Error::SaveToFile(err) => write!(f, "Failed to save file: {}", err),
        }
    }
}

async fn save_body_to_file(body: hyper::Body, file_path: &PathBuf) -> Result<u64, io::Error> {
    let mut file = File::create(&file_path)?;
    let buf = hyper::body::aggregate(body).await.unwrap();
    let mut reader = buf.reader();
    io::copy(&mut reader, &mut file)
}
