use crate::app_config;
use crate::app_config::set_file_permissions;
use crate::app_config::AppConfig;
use crate::common::account_archive;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub account_name: String,
}

pub fn export(config: Config) -> Result<(), Error> {
    let accounts = AppConfig::list_accounts().map_err(Error::AppConfig)?;
    err_if_account_not_found(&accounts, &config.account_name)?;

    let app_cfg = AppConfig::init_account(&config.account_name).map_err(Error::AppConfig)?;
    let account_path = app_cfg.account_base_path();

    let archive_name = format!("gdrive_export-{}.tar", normalize_name(&config.account_name));
    let archive_path = PathBuf::from(&archive_name);
    account_archive::create(&account_path, &archive_path).map_err(Error::CreateArchive)?;

    if let Err(err) = set_file_permissions(&archive_path) {
        eprintln!("Warning: Failed to set permissions on archive: {}", err);
    }

    println!(
        "Exported account '{}' to {}",
        config.account_name, archive_name
    );

    Ok(())
}

#[derive(Debug)]
pub enum Error {
    AppConfig(app_config::Error),
    AccountNotFound(String),
    CreateArchive(account_archive::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::AppConfig(e) => write!(f, "{}", e),
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
