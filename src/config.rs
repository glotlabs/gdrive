use serde::Deserialize;
use serde::Serialize;
use std::fs;
use std::io;
use std::path::PathBuf;

const SYSTEM_CONFIG_DIR_NAME: &str = ".config";
const BASE_PATH_DIR_NAME: &str = "gdrive3";
const ACCOUNT_CONFIG_NAME: &str = "account.json";
const SECRET_CONFIG_NAME: &str = "secret.json";
const TOKENS_CONFIG_NAME: &str = "tokens.json";

#[derive(Debug, Clone)]
pub struct Config {
    pub base_path: PathBuf,
    pub account: Account,
}

pub fn add_account(
    account_name: &str,
    secret: &Secret,
    tokens_path: &PathBuf,
) -> Result<Config, Error> {
    let config = Config::init_account(account_name)?;
    config.save_secret(secret)?;
    fs::copy(tokens_path, config.tokens_path()).map_err(Error::CopyTokens)?;
    Ok(config)
}

pub fn switch_account(config: &Config) -> Result<(), Error> {
    config.save_account_config()
}

impl Config {
    pub fn load_current_account() -> Result<Config, Error> {
        let base_path = Config::default_base_path()?;
        let account_config = Config::load_account_config()?;
        let account = Account::new(&account_config.current);
        let config = Config { base_path, account };
        Ok(config)
    }

    pub fn init_account(account_name: &str) -> Result<Config, Error> {
        let base_path = Config::default_base_path()?;
        let account = Account::new(account_name);

        let config = Config { base_path, account };
        config.create_account_dir()?;

        Ok(config)
    }

    pub fn save_secret(&self, secret: &Secret) -> Result<(), Error> {
        let content = serde_json::to_string_pretty(&secret).map_err(Error::SerializeSecret)?;
        fs::write(&self.secret_path(), content).map_err(Error::WriteSecret)?;
        Ok(())
    }

    pub fn load_secret(&self) -> Result<Secret, Error> {
        let content = fs::read_to_string(&self.secret_path()).map_err(Error::ReadSecret)?;
        serde_json::from_str(&content).map_err(Error::DeserializeSecret)
    }

    pub fn load_account_config() -> Result<AccountConfig, Error> {
        let base_path = Config::default_base_path()?;
        let content = fs::read_to_string(base_path.join(ACCOUNT_CONFIG_NAME))
            .map_err(Error::ReadAccountConfig)?;
        serde_json::from_str(&content).map_err(Error::DeserializeAccountConfig)
    }

    pub fn save_account_config(&self) -> Result<(), Error> {
        let account_config = AccountConfig {
            current: self.account.name.clone(),
        };

        let base_path = Config::default_base_path()?;
        let content =
            serde_json::to_string_pretty(&account_config).map_err(Error::SerializeAccountConfig)?;
        fs::write(self.account_config_path(), content).map_err(Error::WriteAccountConfig)?;
        Ok(())
    }

    pub fn account_config_path(&self) -> PathBuf {
        self.base_path.join(ACCOUNT_CONFIG_NAME)
    }

    pub fn account_base_path(&self) -> PathBuf {
        self.base_path.join(&self.account.name)
    }

    pub fn secret_path(&self) -> PathBuf {
        self.account_base_path().join(SECRET_CONFIG_NAME)
    }

    pub fn tokens_path(&self) -> PathBuf {
        self.account_base_path().join(TOKENS_CONFIG_NAME)
    }

    fn default_base_path() -> Result<PathBuf, Error> {
        let home_path = home::home_dir().ok_or(Error::HomeDirNotFound)?;
        let base_path = home_path
            .join(SYSTEM_CONFIG_DIR_NAME)
            .join(BASE_PATH_DIR_NAME);
        Ok(base_path)
    }

    fn create_account_dir(&self) -> Result<(), Error> {
        fs::create_dir_all(&self.account_base_path()).map_err(Error::CreateConfigDir)?;
        Ok(())
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AccountConfig {
    pub current: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Account {
    pub name: String,
}

impl Account {
    pub fn new(name: &str) -> Account {
        Account {
            name: name.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secret {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug)]
pub enum Error {
    HomeDirNotFound,
    CreateConfigDir(io::Error),
    ReadAccountConfig(io::Error),
    ParseAccountConfig(serde_json::Error),
    SerializeAccountConfig(serde_json::Error),
    WriteAccountConfig(io::Error),
    SerializeSecret(serde_json::Error),
    WriteSecret(io::Error),
    ReadSecret(io::Error),
    DeserializeSecret(serde_json::Error),
    DeserializeAccountConfig(serde_json::Error),
    CopyTokens(io::Error),
}
