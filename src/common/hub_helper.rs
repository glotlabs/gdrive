use crate::app_config;
use crate::app_config::AppConfig;
use crate::hub::Auth;
use crate::hub::Hub;
use std::error;
use std::fmt;
use std::fmt::Display;
use std::fmt::Formatter;
use std::io;

pub async fn get_hub() -> Result<Hub, Error> {
    let app_cfg = AppConfig::load_current_account().map_err(Error::AppConfig)?;
    let secret = app_cfg.load_secret().map_err(Error::AppConfig)?;
    let auth = Auth::new(&secret, &app_cfg.tokens_path())
        .await
        .map_err(Error::Auth)?;

    let hub = Hub::new(auth).await;

    Ok(hub)
}

#[derive(Debug)]
pub enum Error {
    AppConfig(app_config::Error),
    Auth(io::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::AppConfig(err) => write!(f, "{}", err),
            Error::Auth(err) => write!(f, "Auth error: {}", err),
        }
    }
}
