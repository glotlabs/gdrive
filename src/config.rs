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

impl Config {
    pub fn init(account_name: &str) -> Result<Config, Error> {
        let base_path = Config::default_base_path()?;
        let account_json_path = &base_path.join(ACCOUNT_CONFIG_NAME);
        let account = Account::init(account_json_path, account_name)?;

        let config = Config { base_path, account };
        config.create_account_dir()?;

        Ok(config)
    }

    pub fn load_default_account() -> Result<Config, Error> {
        let base_path = Config::default_base_path()?;
        let account_json_path = &base_path.join(ACCOUNT_CONFIG_NAME);
        let account = Account::read_from_path(&account_json_path)?;

        Ok(Config { base_path, account })
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
pub struct Account {
    pub name: String,
}

impl Account {
    pub fn init(path: &PathBuf, name: &str) -> Result<Account, Error> {
        let account = Account {
            name: name.to_string(),
        };

        let content =
            serde_json::to_string_pretty(&account).map_err(Error::SerializeAccountConfig)?;
        fs::write(path, content).map_err(Error::WriteAccountConfig)?;
        Ok(account)
    }

    pub fn new(name: &str) -> Account {
        Account {
            name: name.to_string(),
        }
    }

    pub fn read_from_path(path: &PathBuf) -> Result<Account, Error> {
        let content = fs::read_to_string(path).map_err(Error::ReadAccountConfig)?;
        serde_json::from_str(&content).map_err(Error::ParseAccountConfig)
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
}
