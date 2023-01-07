use crate::app_config;
use crate::app_config::AppConfig;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;

pub fn remove(account_name: &str) -> Result<(), Error> {
    let accounts = AppConfig::list_accounts().map_err(Error::AppConfig)?;
    err_if_account_not_found(&accounts, account_name)?;

    let app_cfg = AppConfig::init_account(account_name).map_err(Error::AppConfig)?;
    app_cfg.remove_account().map_err(Error::AppConfig)?;
    println!("Removed account '{}'", account_name);

    Ok(())
}

#[derive(Debug)]
pub enum Error {
    AppConfig(app_config::Error),
    AccountNotFound(String),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::AppConfig(e) => write!(f, "{}", e),
            Error::AccountNotFound(name) => write!(f, "Account '{}' not found", name),
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
