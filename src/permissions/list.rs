use crate::common::delegate::UploadDelegate;
use crate::common::delegate::UploadDelegateConfig;
use crate::common::file_table;
use crate::common::file_table::FileTable;
use crate::common::hub_helper;
use crate::common::permission;
use crate::files;
use crate::hub::Hub;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::io;

#[derive(Clone, Debug)]
pub struct Config {
    pub file_id: String,
}

pub async fn list(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;
    let delegate_config = UploadDelegateConfig::default();

    files::info::get_file(&hub, &config.file_id)
        .await
        .map_err(Error::GetFile)?;

    let permissions = list_permissions(&hub, delegate_config, &config.file_id)
        .await
        .map_err(Error::ListPermissions)?;

    print_permissions_table(permissions);

    Ok(())
}

fn print_permissions_table(permissions: Vec<google_drive3::api::Permission>) {
    let mut values: Vec<[String; 6]> = vec![];

    for permission in permissions {
        values.push([
            permission.id.unwrap_or_default(),
            permission.type_.unwrap_or_default(),
            permission.role.unwrap_or_default(),
            permission.email_address.unwrap_or_default(),
            permission.domain.unwrap_or_default(),
            files::info::format_bool(permission.allow_file_discovery.unwrap_or_default()),
        ])
    }

    // TODO: rename to Table or something
    let table = FileTable {
        header: ["Id", "Type", "Role", "Email", "Domain", "Discoverable"],
        values,
    };

    let _ = file_table::write(io::stdout(), table, &file_table::DisplayConfig::default());
}

pub async fn list_permissions(
    hub: &Hub,
    delegate_config: UploadDelegateConfig,
    file_id: &str,
) -> Result<Vec<google_drive3::api::Permission>, google_drive3::Error> {
    let mut delegate = UploadDelegate::new(delegate_config);

    let (_, permission_list) = hub
        .permissions()
        .list(file_id)
        .param(
            "fields",
            "permissions(id,role,type,domain,emailAddress,allowFileDiscovery)",
        )
        .add_scope(google_drive3::api::Scope::Full)
        .delegate(&mut delegate)
        .supports_all_drives(true)
        .doit()
        .await?;

    Ok(permission_list.permissions.unwrap_or_default())
}

#[derive(Debug)]
pub enum Error {
    Hub(hub_helper::Error),
    GetFile(google_drive3::Error),
    ListPermissions(google_drive3::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            Error::Hub(err) => write!(f, "{}", err),
            Error::GetFile(err) => {
                write!(f, "Failed to get file: {}", err)
            }
            Error::ListPermissions(err) => {
                write!(f, "Failed to list permissions: {}", err)
            }
        }
    }
}
