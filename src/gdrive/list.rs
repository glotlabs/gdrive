use crate::config;
use crate::config::Config;
use crate::hub;
use std::io;

pub async fn list() -> Result<(), Error> {
    let config = Config::load_current_account().map_err(Error::Config)?;
    let secret = config.load_secret().map_err(Error::Config)?;
    let auth = hub::Auth::new(&secret, &config.tokens_path())
        .await
        .map_err(Error::Auth)?;

    let hub = hub::Hub::new(auth).await;
    let res = hub.files().list().doit().await;
    println!("{:?}", res);
    Ok(())
}

#[derive(Debug)]
pub enum Error {
    Auth(io::Error),
    Config(config::Error),
}
