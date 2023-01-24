use crate::common::delegate::UploadDelegate;
use crate::common::delegate::UploadDelegateConfig;
use crate::common::hub_helper;
use crate::common::permission;
use crate::files;
use crate::hub::Hub;
use crate::permissions;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Clone, Debug)]
pub struct Config {
    pub file_id: String,
    pub action: RevokeAction,
}

pub async fn revoke(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;
    let delegate_config = UploadDelegateConfig::default();

    let file = files::info::get_file(&hub, &config.file_id)
        .await
        .map_err(Error::GetFile)?;

    let permissions =
        permissions::list::list_permissions(&hub, delegate_config.clone(), &config.file_id)
            .await
            .map_err(Error::ListPermissions)?;

    let delete_list = config.action.get_matching_permissions(permissions)?;

    for permission in delete_list {
        if let Err(_) = print_revoke_details(&file, &permission) {
            println!(
                "Revoking permission with id: '{}'",
                permission.id.clone().unwrap_or_default()
            );
        }

        delete_permission(
            &hub,
            delegate_config.clone(),
            &config.file_id,
            &permission.id.clone().unwrap_or_default(),
        )
        .await
        .map_err(|err| Error::DeletePermission(permission.clone(), err))?;
    }

    Ok(())
}

pub async fn delete_permission(
    hub: &Hub,
    delegate_config: UploadDelegateConfig,
    file_id: &str,
    permission_id: &str,
) -> Result<(), google_drive3::Error> {
    let mut delegate = UploadDelegate::new(delegate_config);

    hub.permissions()
        .delete(file_id, &permission_id)
        .param(
            "fields",
            "id,role,type,domain,emailAddress,allowFileDiscovery",
        )
        .add_scope(google_drive3::api::Scope::Full)
        .delegate(&mut delegate)
        .supports_all_drives(true)
        .doit()
        .await?;

    Ok(())
}

#[derive(Debug)]
pub enum Error {
    Hub(hub_helper::Error),
    GetFile(google_drive3::Error),
    ListPermissions(google_drive3::Error),
    DeletePermission(google_drive3::api::Permission, google_drive3::Error),
    PermissionNotFound(String),
    UnknownPermissionType(String),
    UnknownPermissionRole(String),
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
            Error::DeletePermission(permission, err) => {
                write!(
                    f,
                    "Failed to delete permission '{}': {}",
                    permission.clone().id.unwrap_or_default(),
                    err
                )
            }
            Error::PermissionNotFound(id) => {
                write!(f, "Permission '{}' not found", id)
            }
            Error::UnknownPermissionType(type_) => {
                write!(f, "Unknown permission type: '{}'", type_)
            }
            Error::UnknownPermissionRole(role) => write!(f, "Unknown permission role: '{}'", role),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub enum RevokeAction {
    #[default]
    Anyone,
    AllExceptOwner,
    Id(String),
}

impl RevokeAction {
    fn get_matching_permissions(
        &self,
        permissions: Vec<google_drive3::api::Permission>,
    ) -> Result<Vec<google_drive3::api::Permission>, Error> {
        match self {
            RevokeAction::Anyone => {
                // fmt
                Ok(Self::get_permissions_by_type(
                    permissions,
                    permission::Type::Anyone,
                ))
            }

            RevokeAction::AllExceptOwner => {
                // fmt
                Ok(Self::get_permissions_except_role(
                    permissions,
                    permission::Role::Owner,
                ))
            }

            RevokeAction::Id(id) => {
                // fmt
                Self::find_permission_by_id(permissions, id)
                    .map(|p| vec![p])
                    .ok_or_else(|| Error::PermissionNotFound(id.to_string()))
            }
        }
    }

    fn get_permissions_by_type(
        permissions: Vec<google_drive3::api::Permission>,
        type_: permission::Type,
    ) -> Vec<google_drive3::api::Permission> {
        permissions
            .iter()
            .cloned()
            .filter(|p| p.type_ == Some(type_.to_string()))
            .collect()
    }

    fn get_permissions_except_role(
        permissions: Vec<google_drive3::api::Permission>,
        role: permission::Role,
    ) -> Vec<google_drive3::api::Permission> {
        permissions
            .iter()
            .cloned()
            .filter(|p| p.role != Some(role.to_string()))
            .collect()
    }

    fn find_permission_by_id(
        permissions: Vec<google_drive3::api::Permission>,
        id: &str,
    ) -> Option<google_drive3::api::Permission> {
        permissions
            .iter()
            .cloned()
            .find(|p| p.id == Some(id.to_string()))
    }
}

fn print_revoke_details(
    file: &google_drive3::api::File,
    permission: &google_drive3::api::Permission,
) -> Result<(), Error> {
    let type_: permission::Type = permission
        .type_
        .clone()
        .unwrap_or_default()
        .parse()
        .map_err(|_| Error::UnknownPermissionType(permission.type_.clone().unwrap_or_default()))?;

    let role: permission::Role = permission
        .role
        .clone()
        .unwrap_or_default()
        .parse()
        .map_err(|_| Error::UnknownPermissionRole(permission.role.clone().unwrap_or_default()))?;

    if type_.requires_domain() {
        println!(
            "Revoking '{}' permission to {} '{}' for '{}'",
            role,
            type_,
            permission.domain.clone().unwrap_or_default(),
            file.name.clone().unwrap_or_default()
        );
    } else if type_.requires_email() {
        println!(
            "Revoking '{}' permission to '{}' with email '{}' for '{}'",
            role,
            type_,
            permission.email_address.clone().unwrap_or_default(),
            file.name.clone().unwrap_or_default()
        );
    } else {
        println!(
            "Revoking '{}' permission to '{}' for '{}'",
            role,
            type_,
            file.name.clone().unwrap_or_default()
        );
    }

    Ok(())
}
