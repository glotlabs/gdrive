use std::error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs;
use std::path::PathBuf;

pub struct FileInfo {
    pub name: String,
    pub mime_type: mime::Mime,
    pub parents: Option<Vec<String>>,
    pub size: u64,
}

pub struct Config {
    pub file_path: PathBuf,
    pub mime_type: Option<mime::Mime>,
    pub parents: Option<Vec<String>>,
}

impl FileInfo {
    pub fn from_file(file: &fs::File, config: &Config) -> Result<FileInfo, Error> {
        let file_name = config
            .file_path
            .file_name()
            .map(|s| s.to_string_lossy().to_string())
            .ok_or(Error::InvalidFilePath(config.file_path.clone()))?;

        let file_size = file.metadata().map(|m| m.len()).unwrap_or(0);

        let mime_type = config.mime_type.clone().unwrap_or_else(|| {
            mime_guess::from_path(&config.file_path)
                .first()
                .unwrap_or(mime::APPLICATION_OCTET_STREAM)
        });

        Ok(FileInfo {
            name: file_name,
            mime_type,
            parents: config.parents.clone(),
            size: file_size,
        })
    }
}

#[derive(Debug)]
pub enum Error {
    InvalidFilePath(PathBuf),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidFilePath(path) => write!(f, "Invalid file path: {}", path.display()),
        }
    }
}
