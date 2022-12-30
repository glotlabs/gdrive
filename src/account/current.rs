use crate::config;
use crate::config::Config;

pub fn current() -> Result<(), config::Error> {
    let accounts = Config::list_accounts()?;

    if accounts.is_empty() {
        println!("No accounts found");
        println!("Use `gdrive account add` to add an account.");
    } else {
        let config = Config::load_current_account()?;
        println!("{}", config.account.name);
    }

    Ok(())
}
