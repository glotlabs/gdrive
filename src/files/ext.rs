use crate::common::drive_file::{MIME_TYPE_DRIVE_FOLDER, MIME_TYPE_DRIVE_SHORTCUT};

pub trait FileExtension {
    fn is_directory(&self) -> bool;
    fn is_binary(&self) -> bool;
    fn is_shortcut(&self) -> bool;
}

impl FileExtension for google_drive3::api::File {
    fn is_directory(&self) -> bool {
        self.mime_type == Some(String::from(MIME_TYPE_DRIVE_FOLDER))
    }
    
    fn is_binary(&self) -> bool {
        self.md5_checksum != None
    }
    
    fn is_shortcut(&self) -> bool {
        self.mime_type == Some(String::from(MIME_TYPE_DRIVE_SHORTCUT))
    }
}