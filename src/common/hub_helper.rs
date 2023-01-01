use crate::config;
use crate::config::Config;
use crate::hub::Auth;
use crate::hub::Hub;
use std::error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::io;

pub async fn get_hub() -> Result<Hub, Error> {
    let config = Config::load_current_account().map_err(Error::Config)?;
    let secret = config.load_secret().map_err(Error::Config)?;
    let auth = Auth::new(&secret, &config.tokens_path())
        .await
        .map_err(Error::Auth)?;

    let hub = Hub::new(auth).await;

    Ok(hub)
}

#[derive(Debug)]
pub enum Error {
    Config(config::Error),
    Auth(io::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::Config(err) => write!(f, "Config error: {}", err),
            Error::Auth(err) => write!(f, "Auth error: {}", err),
        }
    }
}
