use mime::Mime;
use std::fmt;
use std::path::PathBuf;

pub const MIME_TYPE_DRIVE_FOLDER: &str = "application/vnd.google-apps.folder";
pub const MIME_TYPE_DRIVE_DOCUMENT: &str = "application/vnd.google-apps.document";
pub const MIME_TYPE_DRIVE_SHORTCUT: &str = "application/vnd.google-apps.shortcut";
pub const MIME_TYPE_DRIVE_SPREADSHEET: &str = "application/vnd.google-apps.spreadsheet";
pub const MIME_TYPE_DRIVE_PRESENTATION: &str = "application/vnd.google-apps.presentation";

pub const EXTENSION_DOC: &str = "doc";
pub const EXTENSION_DOCX: &str = "docx";
pub const EXTENSION_ODT: &str = "odt";
pub const EXTENSION_JPG: &str = "jpg";
pub const EXTENSION_JPEG: &str = "jpeg";
pub const EXTENSION_GIF: &str = "gif";
pub const EXTENSION_PNG: &str = "png";
pub const EXTENSION_RTF: &str = "rtf";
pub const EXTENSION_PDF: &str = "pdf";
pub const EXTENSION_HTML: &str = "html";
pub const EXTENSION_XLS: &str = "xls";
pub const EXTENSION_XLSX: &str = "xlsx";
pub const EXTENSION_CSV: &str = "csv";
pub const EXTENSION_TSV: &str = "tsv";
pub const EXTENSION_ODS: &str = "ods";
pub const EXTENSION_PPT: &str = "ppt";
pub const EXTENSION_PPTX: &str = "pptx";
pub const EXTENSION_ODP: &str = "odp";
pub const EXTENSION_EPUB: &str = "epub";
pub const EXTENSION_TXT: &str = "txt";

pub const MIME_TYPE_DOC: &str = "application/msword";
pub const MIME_TYPE_DOCX: &str =
    "application/vnd.openxmlformats-officedocument.wordprocessingml.document";
pub const MIME_TYPE_ODT: &str = "application/vnd.oasis.opendocument.text";
pub const MIME_TYPE_JPG: &str = "image/jpeg";
pub const MIME_TYPE_JPEG: &str = "image/jpeg";
pub const MIME_TYPE_GIF: &str = "image/gif";
pub const MIME_TYPE_PNG: &str = "image/png";
pub const MIME_TYPE_RTF: &str = "application/rtf";
pub const MIME_TYPE_PDF: &str = "application/pdf";
pub const MIME_TYPE_HTML: &str = "text/html";
pub const MIME_TYPE_XLS: &str = "application/vnd.ms-excel";
pub const MIME_TYPE_XLSX: &str =
    "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";
pub const MIME_TYPE_CSV: &str = "text/csv";
pub const MIME_TYPE_TSV: &str = "text/tab-separated-values";
pub const MIME_TYPE_ODS: &str = "application/vnd.oasis.opendocument.spreadsheet";
pub const MIME_TYPE_PPT: &str = "application/vnd.ms-powerpoint";
pub const MIME_TYPE_PPTX: &str =
    "application/vnd.openxmlformats-officedocument.presentationml.presentation";
pub const MIME_TYPE_ODP: &str = "application/vnd.oasis.opendocument.presentation";
pub const MIME_TYPE_EPUB: &str = "application/epub+zip";
pub const MIME_TYPE_TXT: &str = "text/plain";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocType {
    Document,
    Spreadsheet,
    Presentation,
}

impl DocType {
    const IMPORT_EXTENSION_MAP: &[(FileExtension, DocType)] = &[
        (FileExtension::Doc, DocType::Document),
        (FileExtension::Docx, DocType::Document),
        (FileExtension::Odt, DocType::Document),
        (FileExtension::Jpg, DocType::Document),
        (FileExtension::Jpeg, DocType::Document),
        (FileExtension::Gif, DocType::Document),
        (FileExtension::Png, DocType::Document),
        (FileExtension::Rtf, DocType::Document),
        (FileExtension::Pdf, DocType::Document),
        (FileExtension::Html, DocType::Document),
        (FileExtension::Xls, DocType::Spreadsheet),
        (FileExtension::Xlsx, DocType::Spreadsheet),
        (FileExtension::Csv, DocType::Spreadsheet),
        (FileExtension::Tsv, DocType::Spreadsheet),
        (FileExtension::Ods, DocType::Spreadsheet),
        (FileExtension::Ppt, DocType::Presentation),
        (FileExtension::Pptx, DocType::Presentation),
        (FileExtension::Odp, DocType::Presentation),
    ];

    pub fn from_file_path(path: &PathBuf) -> Option<DocType> {
        let extension = FileExtension::from_path(path)?;

        Self::IMPORT_EXTENSION_MAP
            .iter()
            .find_map(|(ext, doc_type)| {
                if ext == &extension {
                    Some(doc_type.clone())
                } else {
                    None
                }
            })
    }

    pub fn from_mime_type(mime: &str) -> Option<DocType> {
        match mime {
            MIME_TYPE_DRIVE_DOCUMENT => Some(DocType::Document),
            MIME_TYPE_DRIVE_SPREADSHEET => Some(DocType::Spreadsheet),
            MIME_TYPE_DRIVE_PRESENTATION => Some(DocType::Presentation),
            _ => None,
        }
    }

    pub fn supported_import_types() -> Vec<String> {
        Self::IMPORT_EXTENSION_MAP
            .iter()
            .map(|(ext, _)| ext.to_string())
            .collect()
    }

    pub fn default_export_type(&self) -> FileExtension {
        match self {
            DocType::Document => FileExtension::Pdf,
            DocType::Spreadsheet => FileExtension::Csv,
            DocType::Presentation => FileExtension::Pdf,
        }
    }

    pub fn can_export_to(&self, extension: &FileExtension) -> bool {
        self.supported_export_types().contains(extension)
    }

    pub fn supported_export_types(&self) -> Vec<FileExtension> {
        match self {
            DocType::Document => vec![
                FileExtension::Pdf,
                FileExtension::Odt,
                FileExtension::Docx,
                FileExtension::Epub,
                FileExtension::Rtf,
                FileExtension::Txt,
                FileExtension::Html,
            ],

            DocType::Spreadsheet => vec![
                FileExtension::Csv,
                FileExtension::Tsv,
                FileExtension::Ods,
                FileExtension::Xlsx,
                FileExtension::Pdf,
            ],

            DocType::Presentation => vec![
                FileExtension::Pdf,
                FileExtension::Pptx,
                FileExtension::Odp,
                FileExtension::Txt,
            ],
        }
    }

    pub fn mime(&self) -> Option<Mime> {
        match self {
            DocType::Document => MIME_TYPE_DRIVE_DOCUMENT.parse().ok(),
            DocType::Spreadsheet => MIME_TYPE_DRIVE_SPREADSHEET.parse().ok(),
            DocType::Presentation => MIME_TYPE_DRIVE_PRESENTATION.parse().ok(),
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileExtension {
    Doc,
    Docx,
    Odt,
    Jpg,
    Jpeg,
    Gif,
    Png,
    Rtf,
    Pdf,
    Html,
    Xls,
    Xlsx,
    Csv,
    Tsv,
    Ods,
    Ppt,
    Pptx,
    Odp,
    Epub,
    Txt,
}

impl fmt::Display for FileExtension {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FileExtension::Doc => write!(f, "{}", EXTENSION_DOC),
            FileExtension::Docx => write!(f, "{}", EXTENSION_DOCX),
            FileExtension::Odt => write!(f, "{}", EXTENSION_ODT),
            FileExtension::Jpg => write!(f, "{}", EXTENSION_JPG),
            FileExtension::Jpeg => write!(f, "{}", EXTENSION_JPEG),
            FileExtension::Gif => write!(f, "{}", EXTENSION_GIF),
            FileExtension::Png => write!(f, "{}", EXTENSION_PNG),
            FileExtension::Rtf => write!(f, "{}", EXTENSION_RTF),
            FileExtension::Pdf => write!(f, "{}", EXTENSION_PDF),
            FileExtension::Html => write!(f, "{}", EXTENSION_HTML),
            FileExtension::Xls => write!(f, "{}", EXTENSION_XLS),
            FileExtension::Xlsx => write!(f, "{}", EXTENSION_XLSX),
            FileExtension::Csv => write!(f, "{}", EXTENSION_CSV),
            FileExtension::Tsv => write!(f, "{}", EXTENSION_TSV),
            FileExtension::Ods => write!(f, "{}", EXTENSION_ODS),
            FileExtension::Ppt => write!(f, "{}", EXTENSION_PPT),
            FileExtension::Pptx => write!(f, "{}", EXTENSION_PPTX),
            FileExtension::Odp => write!(f, "{}", EXTENSION_ODP),
            FileExtension::Epub => write!(f, "{}", EXTENSION_EPUB),
            FileExtension::Txt => write!(f, "{}", EXTENSION_TXT),
        }
    }
}

impl FileExtension {
    pub fn from_path(path: &PathBuf) -> Option<FileExtension> {
        let extension = path.extension()?.to_str()?;

        match extension {
            EXTENSION_DOC => Some(FileExtension::Doc),
            EXTENSION_DOCX => Some(FileExtension::Docx),
            EXTENSION_ODT => Some(FileExtension::Odt),
            EXTENSION_JPG => Some(FileExtension::Jpg),
            EXTENSION_JPEG => Some(FileExtension::Jpeg),
            EXTENSION_GIF => Some(FileExtension::Gif),
            EXTENSION_PNG => Some(FileExtension::Png),
            EXTENSION_RTF => Some(FileExtension::Rtf),
            EXTENSION_PDF => Some(FileExtension::Pdf),
            EXTENSION_HTML => Some(FileExtension::Html),
            EXTENSION_XLS => Some(FileExtension::Xls),
            EXTENSION_XLSX => Some(FileExtension::Xlsx),
            EXTENSION_CSV => Some(FileExtension::Csv),
            EXTENSION_TSV => Some(FileExtension::Tsv),
            EXTENSION_ODS => Some(FileExtension::Ods),
            EXTENSION_PPT => Some(FileExtension::Ppt),
            EXTENSION_PPTX => Some(FileExtension::Pptx),
            EXTENSION_ODP => Some(FileExtension::Odp),
            EXTENSION_EPUB => Some(FileExtension::Epub),
            EXTENSION_TXT => Some(FileExtension::Txt),
            _ => None,
        }
    }

    pub fn get_export_mime(&self) -> Option<Mime> {
        match self {
            FileExtension::Doc => MIME_TYPE_DOC.parse().ok(),
            FileExtension::Docx => MIME_TYPE_DOCX.parse().ok(),
            FileExtension::Odt => MIME_TYPE_ODT.parse().ok(),
            FileExtension::Jpg => MIME_TYPE_JPG.parse().ok(),
            FileExtension::Jpeg => MIME_TYPE_JPEG.parse().ok(),
            FileExtension::Gif => MIME_TYPE_GIF.parse().ok(),
            FileExtension::Png => MIME_TYPE_PNG.parse().ok(),
            FileExtension::Rtf => MIME_TYPE_RTF.parse().ok(),
            FileExtension::Pdf => MIME_TYPE_PDF.parse().ok(),
            FileExtension::Html => MIME_TYPE_HTML.parse().ok(),
            FileExtension::Xls => MIME_TYPE_XLS.parse().ok(),
            FileExtension::Xlsx => MIME_TYPE_XLSX.parse().ok(),
            FileExtension::Csv => MIME_TYPE_CSV.parse().ok(),
            FileExtension::Tsv => MIME_TYPE_TSV.parse().ok(),
            FileExtension::Ods => MIME_TYPE_ODS.parse().ok(),
            FileExtension::Ppt => MIME_TYPE_PPT.parse().ok(),
            FileExtension::Pptx => MIME_TYPE_PPTX.parse().ok(),
            FileExtension::Odp => MIME_TYPE_ODP.parse().ok(),
            FileExtension::Epub => MIME_TYPE_EPUB.parse().ok(),
            FileExtension::Txt => MIME_TYPE_TXT.parse().ok(),
        }
    }
}

pub fn is_directory(file: &google_drive3::api::File) -> bool {
    file.mime_type == Some(String::from(MIME_TYPE_DRIVE_FOLDER))
}

pub fn is_binary(file: &google_drive3::api::File) -> bool {
    file.md5_checksum != None
}

pub fn is_shortcut(file: &google_drive3::api::File) -> bool {
    file.mime_type == Some(String::from(MIME_TYPE_DRIVE_SHORTCUT))
}
