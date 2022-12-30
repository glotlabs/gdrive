use crate::config;
use crate::config::Config;

pub fn switch(account_name: &str) -> Result<(), Error> {
    let accounts = Config::list_accounts().map_err(Error::Config)?;

    if !accounts.contains(&account_name.to_string()) {
        println!("Account '{}' not found", account_name);
    } else {
        let config = Config::init_account(account_name).map_err(Error::Config)?;
        config::switch_account(&config).map_err(Error::Config)?;
        println!("Switched to account '{}'", account_name);
    }

    Ok(())
}

#[derive(Debug)]
pub enum Error {
    Config(config::Error),
}
