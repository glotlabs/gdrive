use mime::Mime;
use std::fmt;
use std::path::PathBuf;

pub const MIME_TYPE_FOLDER: &str = "application/vnd.google-apps.folder";
pub const MIME_TYPE_DOCUMENT: &str = "application/vnd.google-apps.document";
pub const MIME_TYPE_SPREADSHEET: &str = "application/vnd.google-apps.spreadsheet";
pub const MIME_TYPE_PRESENTATION: &str = "application/vnd.google-apps.presentation";

#[derive(Debug, Clone)]
pub enum DocType {
    Document,
    Spreadsheet,
    Presentation,
}

impl DocType {
    const EXTENSION_MAP: &[(&'static str, DocType)] = &[
        ("doc", DocType::Document),
        ("docx", DocType::Document),
        ("odt", DocType::Document),
        ("jpg", DocType::Document),
        ("jpeg", DocType::Document),
        ("gif", DocType::Document),
        ("png", DocType::Document),
        ("rtf", DocType::Document),
        ("pdf", DocType::Document),
        ("html", DocType::Document),
        ("xls", DocType::Spreadsheet),
        ("xlsx", DocType::Spreadsheet),
        ("csv", DocType::Spreadsheet),
        ("tsv", DocType::Spreadsheet),
        ("ods", DocType::Spreadsheet),
        ("ppt", DocType::Presentation),
        ("pptx", DocType::Presentation),
        ("odp", DocType::Presentation),
    ];

    pub fn supported_types() -> Vec<String> {
        Self::EXTENSION_MAP
            .iter()
            .map(|(ext, _)| ext.to_string())
            .collect()
    }

    pub fn from_file_path(path: &PathBuf) -> Option<DocType> {
        let extension = path.extension()?.to_str()?;

        Self::EXTENSION_MAP.iter().find_map(|(ext, doc_type)| {
            if ext == &extension {
                Some(doc_type.clone())
            } else {
                None
            }
        })
    }

    pub fn mime(&self) -> Option<Mime> {
        match self {
            DocType::Document => MIME_TYPE_DOCUMENT.parse().ok(),
            DocType::Spreadsheet => MIME_TYPE_SPREADSHEET.parse().ok(),
            DocType::Presentation => MIME_TYPE_PRESENTATION.parse().ok(),
        }
    }
}

impl fmt::Display for DocType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DocType::Document => write!(f, "document"),
            DocType::Spreadsheet => write!(f, "spreadsheet"),
            DocType::Presentation => write!(f, "presentation"),
        }
    }
}

pub fn is_directory(file: &google_drive3::api::File) -> bool {
    file.mime_type == Some(String::from(MIME_TYPE_FOLDER))
}

pub fn is_binary(file: &google_drive3::api::File) -> bool {
    file.md5_checksum != None
}
