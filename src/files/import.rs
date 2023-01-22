use crate::common::chunk_size::ChunkSize;
use crate::common::delegate::BackoffConfig;
use crate::common::delegate::UploadDelegateConfig;
use crate::common::drive_file;
use crate::common::drive_file::DocType;
use crate::common::file_info;
use crate::common::file_info::FileInfo;
use crate::common::hub_helper;
use crate::files;
use crate::files::info::DisplayConfig;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct Config {
    pub file_path: PathBuf,
    pub parents: Option<Vec<String>>,
    pub print_only_id: bool,
}

pub async fn import(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;

    let delegate_config = UploadDelegateConfig {
        chunk_size: ChunkSize::default().in_bytes(),
        backoff_config: BackoffConfig {
            max_retries: 100,
            min_sleep: Duration::from_secs(1),
            max_sleep: Duration::from_secs(30),
        },
        print_chunk_errors: false,
        print_chunk_info: false,
    };

    let doc_type =
        drive_file::DocType::from_file_path(&config.file_path).ok_or(Error::UnsupportedFileType)?;
    let mime_type = doc_type.mime().ok_or(Error::GetMime(doc_type.clone()))?;

    let file = fs::File::open(&config.file_path)
        .map_err(|err| Error::OpenFile(config.file_path.clone(), err))?;

    let file_info = FileInfo::from_file(
        &file,
        &file_info::Config {
            file_path: config.file_path.clone(),
            mime_type: Some(mime_type),
            parents: config.parents.clone(),
        },
    )
    .map_err(Error::FileInfo)?;

    let reader = std::io::BufReader::new(file);

    if !config.print_only_id {
        println!("Importing {} as a {}", config.file_path.display(), doc_type);
    }

    let file = files::upload::upload_file(&hub, reader, None, file_info, delegate_config)
        .await
        .map_err(Error::UploadFile)?;

    if config.print_only_id {
        print!("{}", file.id.unwrap_or_default())
    } else {
        println!("File successfully imported");
        let fields = files::info::prepare_fields(&file, &DisplayConfig::default());
        files::info::print_fields(&fields);
    }

    Ok(())
}

#[derive(Debug)]
pub enum Error {
    Hub(hub_helper::Error),
    OpenFile(PathBuf, io::Error),
    FileInfo(file_info::Error),
    UploadFile(google_drive3::Error),
    UnsupportedFileType,
    GetMime(drive_file::DocType),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Hub(err) => write!(f, "{}", err),
            Error::OpenFile(path, err) => {
                write!(f, "Failed to open file '{}': {}", path.display(), err)
            }
            Error::FileInfo(err) => write!(f, "Failed to get file info: {}", err),
            Error::UploadFile(err) => {
                write!(f, "Failed to upload file: {}", err)
            }
            Error::UnsupportedFileType => write!(
                f,
                "Unsupported file type, supported file types: {}",
                DocType::supported_import_types().join(", ")
            ),
            Error::GetMime(doc_type) => write!(
                f,
                "Failed to get mime type from document type: {}",
                doc_type
            ),
        }
    }
}
