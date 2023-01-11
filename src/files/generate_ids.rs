use crate::common::delegate::UploadDelegate;
use crate::common::delegate::UploadDelegateConfig;
use crate::hub::Hub;

pub async fn generate_ids(
    hub: &Hub,
    count: i32,
    delegate_config: UploadDelegateConfig,
) -> Result<Vec<String>, google_drive3::Error> {
    let mut delegate = UploadDelegate::new(delegate_config);

    let (_, ids) = hub
        .files()
        .generate_ids()
        .count(count)
        .add_scope(google_drive3::api::Scope::Full)
        .delegate(&mut delegate)
        .doit()
        .await?;

    Ok(ids.ids.unwrap_or_default())
}
