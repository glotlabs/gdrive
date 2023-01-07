use crate::app_config;
use crate::app_config::AppConfig;

pub fn remove(account_name: &str) -> Result<(), app_config::Error> {
    let accounts = AppConfig::list_accounts()?;

    if !accounts.contains(&account_name.to_string()) {
        println!("Account '{}' not found", account_name);
    } else {
        let app_cfg = AppConfig::init_account(account_name)?;
        app_cfg.remove_account()?;
        println!("Removed account '{}'", account_name);
    }

    Ok(())
}
