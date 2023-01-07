use crate::app_config;
use crate::app_config::AppConfig;

pub fn current() -> Result<(), app_config::Error> {
    let accounts = AppConfig::list_accounts()?;

    if accounts.is_empty() {
        println!("No accounts found");
        println!("Use `gdrive account add` to add an account.");
    } else {
        let app_cfg = AppConfig::load_current_account()?;
        println!("{}", app_cfg.account.name);
    }

    Ok(())
}
