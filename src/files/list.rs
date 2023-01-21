use crate::common::drive_file;
use crate::common::file_table;
use crate::common::file_table::FileTable;
use crate::common::hub_helper;
use crate::files;
use crate::files::info::DisplayConfig;
use crate::hub::Hub;
use std::cmp::min;
use std::error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::io;
use std::str::FromStr;

const MAX_PAGE_SIZE: usize = 1000;

pub struct Config {
    pub query: ListQuery,
    pub order_by: ListSortOrder,
    pub max_files: usize,
}

pub async fn list(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;
    let files = list_files(&hub, &config).await?;

    let mut values: Vec<[String; 5]> = vec![];

    for file in files {
        let file_type = simplified_file_type(&file);

        values.push([
            file.id.unwrap_or_default(),
            file.name
                .map(|s| truncate_middle(&s, 41))
                .unwrap_or_default(),
            file_type,
            file.size
                .map(|bytes| files::info::format_bytes(bytes, &DisplayConfig::default()))
                .unwrap_or_default(),
            file.created_time
                .map(files::info::format_date_time)
                .unwrap_or_default(),
        ])
    }

    let table = FileTable {
        header: ["Id", "Name", "Type", "Size", "Created"],
        values,
    };

    let _ = file_table::write(io::stdout(), table, &file_table::DisplayConfig::default());

    Ok(())
}

pub async fn list_files(
    hub: &Hub,
    config: &Config,
) -> Result<Vec<google_drive3::api::File>, Error> {
    let mut collected_files: Vec<google_drive3::api::File> = vec![];
    let mut next_page_token: Option<String> = None;

    loop {
        let max_files = config.max_files - collected_files.len();
        let page_size = min(MAX_PAGE_SIZE, max_files);

        let mut req = hub.files().list();

        if let Some(token) = next_page_token {
            req = req.page_token(&token);
        }

        let (_, file_list) = req
            .page_size(page_size as i32)
            .q(&config.query.to_string())
            .order_by(&config.order_by.to_string())
            .add_scope(google_drive3::api::Scope::Full)
            .param(
                "fields",
                "files(id,name,md5Checksum,mimeType,size,createdTime,parents),nextPageToken",
            )
            .doit()
            .await
            .map_err(Error::ListFiles)?;

        if let Some(mut files) = file_list.files {
            collected_files.append(&mut files);
        }

        next_page_token = file_list.next_page_token;

        if collected_files.len() >= config.max_files || next_page_token.is_none() {
            break;
        }
    }

    let max_files = min(config.max_files, collected_files.len());
    Ok(collected_files[0..max_files].to_vec())
}

#[derive(Debug, Clone, Default)]
pub enum ListQuery {
    #[default]
    RootNotTrashed,
    FilesInFolder {
        folder_id: String,
    },
    Custom(String),
    None,
}

impl FromStr for ListQuery {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Ok(ListQuery::None)
        } else {
            Ok(ListQuery::Custom(s.to_string()))
        }
    }
}

impl Display for ListQuery {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ListQuery::RootNotTrashed => {
                write!(f, "'root' in parents and trashed = false")
            }

            ListQuery::FilesInFolder { folder_id } => {
                write!(f, "'{}' in parents and trashed = false", folder_id)
            }

            ListQuery::Custom(query) => {
                write!(f, "{}", query)
            }

            ListQuery::None => {
                write!(f, "")
            }
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum ListSortOrder {
    #[default]
    FolderModifiedName,
    Custom(String),
}

impl FromStr for ListSortOrder {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Err("Sort by can't be an empty string".to_string())
        } else {
            Ok(ListSortOrder::Custom(s.to_string()))
        }
    }
}

impl fmt::Display for ListSortOrder {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            ListSortOrder::FolderModifiedName => {
                write!(f, "folder,modifiedTime desc,name")
            }

            ListSortOrder::Custom(query) => {
                write!(f, "{}", query)
            }
        }
    }
}

#[derive(Debug)]
pub enum Error {
    Hub(hub_helper::Error),
    ListFiles(google_drive3::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Hub(e) => write!(f, "{}", e),
            Error::ListFiles(e) => write!(f, "Failed to list files: {}", e),
        }
    }
}

fn simplified_file_type(file: &google_drive3::api::File) -> String {
    if drive_file::is_directory(file) {
        String::from("folder")
    } else if drive_file::is_binary(file) {
        String::from("regular")
    } else {
        String::from("document")
    }
}

// Truncates string to given max length, and inserts ellipsis into
// the middle of the string to signify that the string has been truncated
fn truncate_middle(s: &str, max_length: usize) -> String {
    let chars: Vec<char> = s.chars().collect();

    if chars.len() <= max_length {
        return s.to_string();
    }

    let tail_count = max_length / 2;
    let head_count = max_length - tail_count - 1;

    let head: String = chars[0..head_count].iter().collect();
    let tail: String = chars[chars.len() - tail_count..].iter().collect();

    vec![head, tail].join("…")
}
