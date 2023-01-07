use crate::app_config;
use crate::app_config::AppConfig;
use crate::common::account_archive;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub archive_path: PathBuf,
}

pub fn import(config: Config) -> Result<(), Error> {
    let account_name =
        account_archive::get_account_name(&config.archive_path).map_err(Error::ReadAccountName)?;

    let accounts = app_config::list_accounts().map_err(Error::AppConfig)?;
    err_if_account_exists(&accounts, &account_name)?;

    let config_base_path = AppConfig::default_base_path().map_err(Error::AppConfig)?;
    account_archive::unpack(&config.archive_path, &config_base_path).map_err(Error::Unpack)?;

    println!("Imported account '{}'", account_name);

    if !AppConfig::has_current_account() {
        let app_cfg = AppConfig::load_account(&account_name).map_err(Error::AppConfig)?;
        println!("Switched to account '{}'", account_name);
        app_config::switch_account(&app_cfg).map_err(Error::AppConfig)?;
    }

    Ok(())
}

#[derive(Debug)]
pub enum Error {
    AppConfig(app_config::Error),
    AccountExists(String),
    ReadAccountName(account_archive::Error),
    Unpack(account_archive::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::AppConfig(e) => write!(f, "{}", e),
            Error::AccountExists(name) => write!(f, "Account '{}' already exists", name),
            Error::ReadAccountName(e) => write!(f, "{}", e),
            Error::Unpack(e) => write!(f, "{}", e),
        }
    }
}

fn err_if_account_exists(accounts: &[String], account_name: &str) -> Result<(), Error> {
    if accounts.contains(&account_name.to_string()) {
        Err(Error::AccountExists(account_name.to_string()))
    } else {
        Ok(())
    }
}
