use crate::app_config;
use crate::app_config::AppConfig;

pub fn list() -> Result<(), app_config::Error> {
    let accounts = AppConfig::list_accounts()?;

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
