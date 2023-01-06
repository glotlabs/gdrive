use crate::common::tar_helper;
use crate::config;
use crate::config::set_file_permissions;
use crate::config::Config;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::path::PathBuf;

pub fn export(account_name: &str) -> Result<(), Error> {
    let accounts = Config::list_accounts().map_err(Error::Config)?;
    err_if_account_not_found(&accounts, account_name)?;

    let config = Config::init_account(account_name).map_err(Error::Config)?;
    let account_path = config.account_base_path();

    let archive_name = format!("gdrive_export-{}.tar", normalize_name(account_name));
    let archive_path = PathBuf::from(&archive_name);
    tar_helper::archive_dir(&account_path, &archive_path).map_err(Error::CreateArchive)?;

    if let Err(err) = set_file_permissions(&archive_path) {
        eprintln!("Warning: Failed to set permissions on archive: {}", err);
    }

    println!("Exported account '{}' to {}", account_name, archive_name);

    Ok(())
}

#[derive(Debug)]
pub enum Error {
    Config(config::Error),
    AccountNotFound(String),
    CreateArchive(tar_helper::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Config(e) => write!(f, "{}", e),
            Error::AccountNotFound(name) => write!(f, "Account '{}' not found", name),
            Error::CreateArchive(e) => write!(f, "{}", e),
        }
    }
}

fn err_if_account_not_found(accounts: &[String], account_name: &str) -> Result<(), Error> {
    if !accounts.contains(&account_name.to_string()) {
        Err(Error::AccountNotFound(account_name.to_string()))
    } else {
        Ok(())
    }
}

fn normalize_name(account_name: &str) -> String {
    account_name
        .chars()
        .map(|c| if char::is_alphanumeric(c) { c } else { '_' })
        .collect()
}
