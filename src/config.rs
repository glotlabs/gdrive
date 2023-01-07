use serde::Deserialize;
use serde::Serialize;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;
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
    pub fn has_current_account() -> bool {
        if let Ok(base_path) = Config::default_base_path() {
            let account_config_path = base_path.join(ACCOUNT_CONFIG_NAME);
            account_config_path.exists()
        } else {
            false
        }
    }

    pub fn load_current_account() -> Result<Config, Error> {
        let base_path = Config::default_base_path()?;
        let account_config = Config::load_account_config()?;
        let account = Account::new(&account_config.current);
        let config = Config { base_path, account };
        Ok(config)
    }

    pub fn load_account(account_name: &str) -> Result<Config, Error> {
        let base_path = Config::default_base_path()?;
        let account = Account::new(account_name);
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

    pub fn remove_account(&self) -> Result<(), Error> {
        let path = self.account_base_path();
        fs::remove_dir_all(&path).map_err(Error::RemoveAccountDir)?;

        let account_config = Config::load_account_config()?;
        if self.account.name == account_config.current {
            fs::remove_file(self.account_config_path()).map_err(Error::RemoveAccountConfig)?;
        }

        Ok(())
    }

    pub fn list_accounts() -> Result<Vec<String>, Error> {
        let base_path = Config::default_base_path()?;
        let entries = fs::read_dir(base_path).map_err(Error::ListFiles)?;

        let mut accounts: Vec<String> = entries
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_dir())
            .filter(|entry| entry.path().join(TOKENS_CONFIG_NAME).exists())
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .collect();

        accounts.sort();

        Ok(accounts)
    }

    pub fn save_secret(&self, secret: &Secret) -> Result<(), Error> {
        let content = serde_json::to_string_pretty(&secret).map_err(Error::SerializeSecret)?;
        let path = self.secret_path();
        fs::write(&path, content).map_err(Error::WriteSecret)?;

        if let Err(err) = set_file_permissions(&path) {
            eprintln!(
                "Warning: Failed to set file permissions on secrets file: {}",
                err
            );
        }

        Ok(())
    }

    pub fn load_secret(&self) -> Result<Secret, Error> {
        let content = fs::read_to_string(&self.secret_path()).map_err(Error::ReadSecret)?;
        serde_json::from_str(&content).map_err(Error::DeserializeSecret)
    }

    pub fn load_account_config() -> Result<AccountConfig, Error> {
        let base_path = Config::default_base_path()?;
        let account_config_path = base_path.join(ACCOUNT_CONFIG_NAME);
        account_config_path
            .exists()
            .then_some(())
            .ok_or(Error::AccountConfigMissing)?;
        let content = fs::read_to_string(account_config_path).map_err(Error::ReadAccountConfig)?;
        serde_json::from_str(&content).map_err(Error::DeserializeAccountConfig)
    }

    pub fn save_account_config(&self) -> Result<(), Error> {
        let account_config = AccountConfig {
            current: self.account.name.clone(),
        };

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

    pub fn default_base_path() -> Result<PathBuf, Error> {
        let home_path = home::home_dir().ok_or(Error::HomeDirNotFound)?;
        let base_path = home_path
            .join(SYSTEM_CONFIG_DIR_NAME)
            .join(BASE_PATH_DIR_NAME);
        Ok(base_path)
    }

    fn create_account_dir(&self) -> Result<(), Error> {
        let path = self.account_base_path();
        fs::create_dir_all(&path).map_err(Error::CreateConfigDir)?;
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

pub fn set_file_permissions(path: &PathBuf) -> Result<(), io::Error> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }

    Ok(())
}

#[derive(Debug)]
pub enum Error {
    HomeDirNotFound,
    CreateConfigDir(io::Error),
    ReadAccountConfig(io::Error),
    AccountConfigMissing,
    ParseAccountConfig(serde_json::Error),
    SerializeAccountConfig(serde_json::Error),
    WriteAccountConfig(io::Error),
    SerializeSecret(serde_json::Error),
    WriteSecret(io::Error),
    ReadSecret(io::Error),
    DeserializeSecret(serde_json::Error),
    DeserializeAccountConfig(serde_json::Error),
    CopyTokens(io::Error),
    ListFiles(io::Error),
    RemoveAccountDir(io::Error),
    RemoveAccountConfig(io::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::HomeDirNotFound => {
                // fmt
                write!(f, "Home directory not found")
            }

            Error::CreateConfigDir(err) => {
                // fmt
                write!(f, "Failed to create config directory: {}", err)
            }

            Error::ReadAccountConfig(err) => {
                // fmt
                write!(f, "Failed to read account config: {}", err)
            }

            Error::AccountConfigMissing => {
                // fmt
                writeln!(f, "No account has been selected")?;
                writeln!(f, "Use `gdrive account list` to show all accounts.")?;
                write!(f, "Use `gdrive account switch` to select an account.")
            }

            Error::ParseAccountConfig(err) => {
                // fmt
                write!(f, "Failed to parse account config: {}", err)
            }

            Error::SerializeAccountConfig(err) => {
                // fmt
                write!(f, "Failed to serialize account config: {}", err)
            }

            Error::WriteAccountConfig(err) => {
                // fmt
                write!(f, "Failed to write account config: {}", err)
            }

            Error::SerializeSecret(err) => {
                // fmt
                write!(f, "Failed to serialize secret: {}", err)
            }

            Error::WriteSecret(err) => {
                // fmt
                write!(f, "Failed to write secret: {}", err)
            }

            Error::ReadSecret(err) => {
                // fmt
                write!(f, "Failed to read secret: {}", err)
            }

            Error::DeserializeSecret(err) => {
                // fmt
                write!(f, "Failed to deserialize secret: {}", err)
            }

            Error::DeserializeAccountConfig(err) => {
                // fmt
                write!(f, "Failed to deserialize account config: {}", err)
            }

            Error::CopyTokens(err) => {
                // fmt
                write!(f, "Failed to copy tokens: {}", err)
            }

            Error::ListFiles(err) => {
                // fmt
                write!(f, "Failed to list files: {}", err)
            }

            Error::RemoveAccountDir(err) => {
                // fmt
                write!(f, "Failed to remove account directory: {}", err)
            }

            Error::RemoveAccountConfig(err) => {
                // fmt
                write!(f, "Failed to remove account config: {}", err)
            }
        }
    }
}
