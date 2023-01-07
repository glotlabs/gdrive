use crate::common::account_archive;
use crate::config;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub archive_path: PathBuf,
}

// TODO: Ensure config dir exists (import might be their first command)
pub fn import(config: Config) -> Result<(), Error> {
    let account_name =
        account_archive::get_account_name(&config.archive_path).map_err(Error::ReadAccountName)?;

    let accounts = config::Config::list_accounts().map_err(Error::Config)?;
    err_if_account_exists(&accounts, &account_name)?;

    let config_base_path = config::Config::default_base_path().map_err(Error::Config)?;
    account_archive::unpack(&config.archive_path, &config_base_path).map_err(Error::Unpack)?;

    println!("Imported account '{}'", account_name);

    if !config::Config::has_current_account() {
        let cfg = config::Config::load_account(&account_name).map_err(Error::Config)?;
        println!("Switched to account '{}'", account_name);
        config::switch_account(&cfg).map_err(Error::Config)?;
    }

    Ok(())
}

#[derive(Debug)]
pub enum Error {
    Config(config::Error),
    AccountExists(String),
    ReadAccountName(account_archive::Error),
    Unpack(account_archive::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Config(e) => write!(f, "{}", e),
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
