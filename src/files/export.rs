use mime::Mime;

use crate::common::drive_file;
use crate::common::drive_file::DocType;
use crate::common::drive_file::FileExtension;
use crate::common::hub_helper;
use crate::files;
use crate::hub::Hub;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct Config {
    pub file_id: String,
    pub file_path: PathBuf,
}

pub async fn export(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;

    // TODO: err if exist
    // TODO: err if dir

    let file = files::info::get_file(&hub, &config.file_id)
        .await
        .map_err(Error::GetFile)?;

    let drive_mime = file.mime_type.ok_or(Error::MissingDriveMime)?;
    let doc_type = DocType::from_mime_type(&drive_mime)
        .ok_or(Error::UnsupportedDriveMime(drive_mime.clone()))?;

    let extension =
        FileExtension::from_path(&config.file_path).ok_or(Error::UnsupportedExportExtension)?;

    if doc_type.can_export_to(&extension) {
        let mime_type = extension
            .get_export_mime()
            .ok_or(Error::GetFileExtensionMime(extension.clone()))?;

        let body = export_file(&hub, &config.file_id, &mime_type)
            .await
            .map_err(Error::ExportFile)?;

        files::download::save_body_to_file(body, &config.file_path, file.md5_checksum)
            .await
            .map_err(Error::SaveFile)?;

        Ok(())
    } else {
        Err(Error::UnsupportedExportExtension)
    }
}

pub async fn export_file(
    hub: &Hub,
    file_id: &str,
    mime_type: &Mime,
) -> Result<hyper::Body, google_drive3::Error> {
    let response = hub
        .files()
        .export(file_id, &mime_type.to_string())
        .add_scope(google_drive3::api::Scope::Full)
        .doit()
        .await?;

    Ok(response.into_body())
}

#[derive(Debug)]
pub enum Error {
    Hub(hub_helper::Error),
    GetFile(google_drive3::Error),
    ExportFile(google_drive3::Error),
    MissingDriveMime,
    UnsupportedDriveMime(String),
    GetFileExtensionMime(drive_file::FileExtension),
    UnsupportedExportExtension,
    SaveFile(files::download::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Hub(err) => write!(f, "{}", err),
            Error::GetFile(err) => {
                write!(f, "Failed to get file: {}", err)
            }
            Error::ExportFile(err) => {
                write!(f, "Failed to export file: {}", err)
            }
            Error::MissingDriveMime => write!(f, "Drive file does not have a mime type"),
            Error::UnsupportedDriveMime(mime) => {
                write!(f, "Mime type on drive file '{}' is not supported", mime)
            }
            Error::GetFileExtensionMime(doc_type) => write!(
                f,
                "Failed to get mime type from file extension: {}",
                doc_type
            ),
            Error::UnsupportedExportExtension => {
                write!(f, "Export to this file extension is not supported")
            }
            Error::SaveFile(err) => {
                write!(f, "Failed to save file: {}", err)
            }
        }
    }
}
