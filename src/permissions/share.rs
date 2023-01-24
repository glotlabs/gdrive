use crate::common::delegate::UploadDelegate;
use crate::common::delegate::UploadDelegateConfig;
use crate::common::hub_helper;
use crate::common::permission;
use crate::files;
use crate::hub::Hub;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Clone, Debug)]
pub struct Config {
    pub file_id: String,
    pub role: permission::Role,
    pub type_: permission::Type,
    pub discoverable: bool,
    pub email: Option<String>,
    pub domain: Option<String>,
}

pub async fn share(config: Config) -> Result<(), Error> {
    err_if_missing_email(&config)?;
    err_if_missing_domain(&config)?;

    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;
    let delegate_config = UploadDelegateConfig::default();

    let file = files::info::get_file(&hub, &config.file_id)
        .await
        .map_err(Error::GetFile)?;

    print_grant_details(&file, &config);

    create_permission(&hub, delegate_config, &config)
        .await
        .map_err(Error::CreatePermission)?;

    Ok(())
}

pub async fn create_permission(
    hub: &Hub,
    delegate_config: UploadDelegateConfig,
    config: &Config,
) -> Result<google_drive3::api::Permission, google_drive3::Error> {
    let mut delegate = UploadDelegate::new(delegate_config);

    let new_permission = google_drive3::api::Permission {
        role: Some(config.role.to_string()),
        type_: Some(config.type_.to_string()),
        allow_file_discovery: Some(config.discoverable),
        email_address: config.email.clone(),
        domain: config.domain.clone(),
        ..google_drive3::api::Permission::default()
    };

    let (_, permission) = hub
        .permissions()
        .create(new_permission, &config.file_id)
        .param(
            "fields",
            "id,role,type,domain,emailAddress,allowFileDiscovery",
        )
        .add_scope(google_drive3::api::Scope::Full)
        .delegate(&mut delegate)
        .supports_all_drives(true)
        .doit()
        .await?;

    Ok(permission)
}

#[derive(Debug)]
pub enum Error {
    Hub(hub_helper::Error),
    GetFile(google_drive3::Error),
    CreatePermission(google_drive3::Error),
    MissingEmail(permission::Type),
    MissingDomain(permission::Type),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Error::Hub(err) => write!(f, "{}", err),
            Error::GetFile(err) => {
                write!(f, "Failed to get file: {}", err)
            }
            Error::CreatePermission(err) => {
                write!(f, "Failed to share file: {}", err)
            }
            Error::MissingEmail(type_) => {
                write!(
                    f,
                    "Email is required for permission type '{}'. Use the --email option",
                    type_
                )
            }
            Error::MissingDomain(type_) => {
                write!(
                    f,
                    "Domain is required for permission type '{}'. Use the --domain option",
                    type_
                )
            }
        }
    }
}

fn err_if_missing_email(config: &Config) -> Result<(), Error> {
    if config.type_.requires_email() && config.email.is_none() {
        return Err(Error::MissingEmail(config.type_.clone()));
    }

    Ok(())
}

fn err_if_missing_domain(config: &Config) -> Result<(), Error> {
    if config.type_.requires_domain() && config.domain.is_none() {
        return Err(Error::MissingDomain(config.type_.clone()));
    }

    Ok(())
}

fn print_grant_details(file: &google_drive3::api::File, config: &Config) {
    if config.type_.requires_domain() {
        println!(
            "Granting '{}' permission to {} '{}' for '{}'",
            config.role,
            config.type_,
            config.domain.clone().unwrap_or_default(),
            file.name.clone().unwrap_or_default()
        );
    } else if config.type_.requires_email() {
        println!(
            "Granting '{}' permission to '{}' with email '{}' for '{}'",
            config.role,
            config.type_,
            config.email.clone().unwrap_or_default(),
            file.name.clone().unwrap_or_default()
        );
    } else {
        println!(
            "Granting '{}' permission to '{}' for '{}'",
            config.role,
            config.type_,
            file.name.clone().unwrap_or_default()
        );
    }
}
