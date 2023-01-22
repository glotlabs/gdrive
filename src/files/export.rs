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
    pub existing_file_action: ExistingFileAction,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ExistingFileAction {
    Abort,
    Overwrite,
}

pub async fn export(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;

    err_if_file_exists(&config)?;

    let file = files::info::get_file(&hub, &config.file_id)
        .await
        .map_err(Error::GetFile)?;

    let drive_mime = file.mime_type.ok_or(Error::MissingDriveMime)?;
    let doc_type = DocType::from_mime_type(&drive_mime)
        .ok_or(Error::UnsupportedDriveMime(drive_mime.clone()))?;

    let extension = FileExtension::from_path(&config.file_path)
        .ok_or(Error::UnsupportedExportExtension(doc_type.clone()))?;

    err_if_unsupported(&doc_type, &extension)?;

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
    FileExists(PathBuf),
    GetFile(google_drive3::Error),
    ExportFile(google_drive3::Error),
    MissingDriveMime,
    UnsupportedDriveMime(String),
    GetFileExtensionMime(drive_file::FileExtension),
    UnsupportedExportExtension(DocType),
    SaveFile(files::download::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Hub(err) => write!(f, "{}", err),
            Error::FileExists(path) => {
                write!(
                    f,
                    "File '{}' already exists, use --overwrite to overwrite it",
                    path.display()
                )
            }
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
            Error::UnsupportedExportExtension(doc_type) => {
                let supported_types = doc_type
                    .supported_export_types()
                    .iter()
                    .map(|ext| ext.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");

                write!(
                    f,
                    "Export of a {} to this file type is not supported, supported file types are: {}",
                    doc_type,
                    supported_types
                )
            }
            Error::SaveFile(err) => {
                write!(f, "Failed to save file: {}", err)
            }
        }
    }
}

fn err_if_file_exists(config: &Config) -> Result<(), Error> {
    if config.file_path.exists() && config.existing_file_action == ExistingFileAction::Abort {
        Err(Error::FileExists(config.file_path.clone()))
    } else {
        Ok(())
    }
}

fn err_if_unsupported(doc_type: &DocType, extension: &FileExtension) -> Result<(), Error> {
    if !doc_type.can_export_to(extension) {
        Err(Error::UnsupportedExportExtension(doc_type.clone()))
    } else {
        Ok(())
    }
}
