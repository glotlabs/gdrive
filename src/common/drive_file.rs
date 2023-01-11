pub const MIME_TYPE_FOLDER: &str = "application/vnd.google-apps.folder";

pub fn is_directory(file: &google_drive3::api::File) -> bool {
    file.mime_type == Some(String::from(MIME_TYPE_FOLDER))
}

pub fn is_binary(file: &google_drive3::api::File) -> bool {
    file.md5_checksum != None
}
