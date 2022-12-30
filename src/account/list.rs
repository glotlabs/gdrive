use crate::config;
use crate::config::Config;

pub fn list() -> Result<(), Error> {
    let accounts = Config::list_accounts().map_err(Error::Config)?;

    if accounts.is_empty() {
        println!("No accounts found");
        println!("Use `gdrive account add` to add an account.");
    } else {
        for account in accounts {
            println!("{}", account);
        }
    }

    Ok(())
}

#[derive(Debug)]
pub enum Error {
    Config(config::Error),
}
