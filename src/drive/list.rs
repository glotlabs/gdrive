use crate::common::hub_helper;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;

pub async fn list() -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;
    let res = hub.files().list().doit().await;
    println!("{:?}", res);
    Ok(())
}

#[derive(Debug)]
pub enum Error {
    Hub(hub_helper::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Hub(e) => write!(f, "{}", e),
        }
    }
}
