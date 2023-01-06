use std::error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs::File;
use std::io;
use std::path::PathBuf;

pub fn archive_dir(src_path: &PathBuf, archive_path: &PathBuf) -> Result<(), Error> {
    err_if_not_exists(src_path)?;
    err_if_not_dir(src_path)?;
    err_if_exists(archive_path)?;

    let archive_file = File::create(archive_path).map_err(Error::CreateFile)?;
    let mut builder = tar::Builder::new(archive_file);

    let src_dir_name = src_path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    builder
        .append_dir_all(&src_dir_name, src_path)
        .map_err(|err| Error::AppendDir(src_path.clone(), err))?;

    builder
        .finish()
        .map_err(|err| Error::FinishArchive(archive_path.clone(), err))?;

    Ok(())
}

#[derive(Debug)]
pub enum Error {
    CreateFile(io::Error),
    PathDoesNotExist(PathBuf),
    PathNotDir(PathBuf),
    PathAlreadyExists(PathBuf),
    AppendDir(PathBuf, io::Error),
    FinishArchive(PathBuf, io::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::CreateFile(err) => {
                // fmt
                write!(f, "Failed to create file: {}", err)
            }

            Error::PathDoesNotExist(path) => {
                // fmt
                write!(f, "'{}' does not exist", path.display())
            }

            Error::PathNotDir(path) => {
                // fmt
                write!(f, "'{}' is not a directory", path.display())
            }

            Error::PathAlreadyExists(path) => {
                // fmt
                write!(f, "'{}' already exists", path.display())
            }

            Error::AppendDir(path, err) => {
                // fmt
                write!(f, "Failed to add {} to archive: {}", path.display(), err)
            }

            Error::FinishArchive(path, err) => {
                // fmt
                write!(f, "Failed to create archive '{}': {}", path.display(), err)
            }
        }
    }
}

fn err_if_not_exists(path: &PathBuf) -> Result<(), Error> {
    if !path.exists() {
        Err(Error::PathDoesNotExist(path.clone()))
    } else {
        Ok(())
    }
}

fn err_if_not_dir(path: &PathBuf) -> Result<(), Error> {
    if !path.is_dir() {
        Err(Error::PathNotDir(path.clone()))
    } else {
        Ok(())
    }
}

fn err_if_exists(path: &PathBuf) -> Result<(), Error> {
    if path.exists() {
        Err(Error::PathAlreadyExists(path.clone()))
    } else {
        Ok(())
    }
}
