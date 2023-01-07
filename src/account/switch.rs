use crate::app_config;
use crate::app_config::AppConfig;

pub fn switch(account_name: &str) -> Result<(), app_config::Error> {
    let accounts = AppConfig::list_accounts()?;

    if !accounts.contains(&account_name.to_string()) {
        println!("Account '{}' not found", account_name);
    } else {
        let app_cfg = AppConfig::init_account(account_name)?;
        app_config::switch_account(&app_cfg)?;
        println!("Switched to account '{}'", account_name);
    }

    Ok(())
}
