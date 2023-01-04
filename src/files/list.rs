use crate::common::hub_helper;
use std::cmp::min;
use std::error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::str::FromStr;

const MAX_PAGE_SIZE: usize = 1000;

pub struct Config {
    pub query: ListQuery,
    pub order_by: ListSortOrder,
    pub max_files: usize,
}

pub async fn list(config: Config) -> Result<(), Error> {
    let files = list_files(&config).await?;

    for file in files {
        println!("{}", file.name.unwrap_or_default());
    }
    Ok(())
}

pub async fn list_files(config: &Config) -> Result<Vec<google_drive3::api::File>, Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;

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
}

impl FromStr for ListQuery {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Err("Query can't be an empty string".to_string())
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
                write!(f, "'{}' in parents", folder_id)
            }

            ListQuery::Custom(query) => {
                write!(f, "{}", query)
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
