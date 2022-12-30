use crate::config;
use crate::config::Config;

pub fn remove(account_name: &str) -> Result<(), config::Error> {
    let accounts = Config::list_accounts()?;

    if !accounts.contains(&account_name.to_string()) {
        println!("Account '{}' not found", account_name);
    } else {
        let config = Config::init_account(account_name)?;
        config.remove_account()?;
        println!("Removed account '{}'", account_name);
    }

    Ok(())
}
