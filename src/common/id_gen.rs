use crate::common::delegate::UploadDelegateConfig;
use crate::files::generate_ids;
use crate::hub::Hub;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;

pub struct IdGen<'a> {
    hub: &'a Hub,
    delegate_config: UploadDelegateConfig,
    ids: Vec<String>,
}

impl<'a> IdGen<'a> {
    pub fn new(hub: &'a Hub, delegate_config: &UploadDelegateConfig) -> Self {
        Self {
            hub,
            delegate_config: delegate_config.clone(),
            ids: Vec::new(),
        }
    }

    pub async fn next(&mut self) -> Result<String, Error> {
        match self.ids.pop() {
            Some(id) => {
                // fmt
                Ok(id)
            }

            None => {
                self.ids = self.generate_ids().await?;
                let id = self.ids.pop().ok_or(Error::OutOfIds)?;
                Ok(id)
            }
        }
    }

    async fn generate_ids(&self) -> Result<Vec<String>, Error> {
        generate_ids::generate_ids(self.hub, 1000, self.delegate_config.clone())
            .await
            .map_err(Error::GenerateIds)
    }
}

#[derive(Debug)]
pub enum Error {
    GenerateIds(google_drive3::Error),
    OutOfIds,
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::GenerateIds(err) => {
                write!(f, "Failed generate id's: {}", err)
            }

            Error::OutOfIds => {
                write!(f, "No more id's available")
            }
        }
    }
}
