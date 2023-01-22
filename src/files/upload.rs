use crate::common::chunk_size::ChunkSize;
use crate::common::delegate::BackoffConfig;
use crate::common::delegate::UploadDelegate;
use crate::common::delegate::UploadDelegateConfig;
use crate::common::file_info;
use crate::common::file_info::FileInfo;
use crate::common::file_tree;
use crate::common::file_tree::FileTree;
use crate::common::hub_helper;
use crate::common::id_gen::IdGen;
use crate::files;
use crate::files::info::DisplayConfig;
use crate::files::mkdir;
use crate::hub::Hub;
use human_bytes::human_bytes;
use mime::Mime;
use std::error;
use std::fmt::Display;
use std::fmt::Formatter;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::Duration;

pub struct Config {
    pub file_path: PathBuf,
    pub mime_type: Option<Mime>,
    pub parents: Option<Vec<String>>,
    pub chunk_size: ChunkSize,
    pub print_chunk_errors: bool,
    pub print_chunk_info: bool,
    pub upload_directories: bool,
    pub print_only_id: bool,
}

pub async fn upload(config: Config) -> Result<(), Error> {
    let hub = hub_helper::get_hub().await.map_err(Error::Hub)?;

    let delegate_config = UploadDelegateConfig {
        chunk_size: config.chunk_size.in_bytes(),
        backoff_config: BackoffConfig {
            max_retries: 100000,
            min_sleep: Duration::from_secs(1),
            max_sleep: Duration::from_secs(60),
        },
        print_chunk_errors: config.print_chunk_errors,
        print_chunk_info: config.print_chunk_info,
    };

    err_if_directory(&config.file_path, &config)?;

    if config.file_path.is_dir() {
        upload_directory(&hub, &config, delegate_config).await?;
    } else {
        upload_regular(&hub, &config, delegate_config).await?;
    }

    Ok(())
}

pub async fn upload_regular(
    hub: &Hub,
    config: &Config,
    delegate_config: UploadDelegateConfig,
) -> Result<(), Error> {
    let file = fs::File::open(&config.file_path)
        .map_err(|err| Error::OpenFile(config.file_path.clone(), err))?;

    let file_info = FileInfo::from_file(
        &file,
        &file_info::Config {
            file_path: config.file_path.clone(),
            mime_type: config.mime_type.clone(),
            parents: config.parents.clone(),
        },
    )
    .map_err(Error::FileInfo)?;

    let reader = std::io::BufReader::new(file);

    if !config.print_only_id {
        println!("Uploading {}", config.file_path.display());
    }

    let file = upload_file(&hub, reader, None, file_info, delegate_config)
        .await
        .map_err(Error::Upload)?;

    if config.print_only_id {
        print!("{}", file.id.unwrap_or_default())
    } else {
        println!("File successfully uploaded");
        let fields = files::info::prepare_fields(&file, &DisplayConfig::default());
        files::info::print_fields(&fields);
    }

    Ok(())
}

pub async fn upload_directory(
    hub: &Hub,
    config: &Config,
    delegate_config: UploadDelegateConfig,
) -> Result<(), Error> {
    let mut ids = IdGen::new(hub, &delegate_config);
    let tree = FileTree::from_path(&config.file_path, &mut ids)
        .await
        .map_err(Error::CreateFileTree)?;

    let tree_info = tree.info();

    if !config.print_only_id {
        println!(
            "Found {} files in {} directories with a total size of {}",
            tree_info.file_count,
            tree_info.folder_count,
            human_bytes(tree_info.total_file_size as f64)
        );
    }

    for folder in &tree.folders() {
        let folder_parents = folder
            .parent
            .as_ref()
            .map(|p| vec![p.drive_id.clone()])
            .or_else(|| config.parents.clone());

        if !config.print_only_id {
            println!(
                "Creating directory '{}' with id: {}",
                folder.relative_path().display(),
                folder.drive_id
            );
        }

        let drive_folder = mkdir::create_directory(
            hub,
            &mkdir::Config {
                id: Some(folder.drive_id.clone()),
                name: folder.name.clone(),
                parents: folder_parents,
                print_only_id: false,
            },
            delegate_config.clone(),
        )
        .await
        .map_err(Error::Mkdir)?;

        if config.print_only_id {
            println!("{}: {}", folder.relative_path().display(), folder.drive_id);
        }

        let folder_id = drive_folder.id.ok_or(Error::DriveFolderMissingId)?;
        let parents = Some(vec![folder_id.clone()]);

        for file in folder.files() {
            let os_file = fs::File::open(&file.path)
                .map_err(|err| Error::OpenFile(config.file_path.clone(), err))?;

            let file_info = file.info(parents.clone());

            if !config.print_only_id {
                println!(
                    "Uploading file '{}' with id: {}",
                    file.relative_path().display(),
                    file.drive_id
                );
            }

            upload_file(
                hub,
                os_file,
                Some(file.drive_id.clone()),
                file_info,
                delegate_config.clone(),
            )
            .await
            .map_err(Error::Upload)?;

            if config.print_only_id {
                println!("{}: {}", file.relative_path().display(), file.drive_id);
            }
        }
    }

    if !config.print_only_id {
        println!(
            "Uploaded {} files in {} directories with a total size of {}",
            tree_info.file_count,
            tree_info.folder_count,
            human_bytes(tree_info.total_file_size as f64)
        );
    }

    Ok(())
}

pub async fn upload_file<RS>(
    hub: &Hub,
    src_file: RS,
    file_id: Option<String>,
    file_info: FileInfo,
    delegate_config: UploadDelegateConfig,
) -> Result<google_drive3::api::File, google_drive3::Error>
where
    RS: google_drive3::client::ReadSeek,
{
    let dst_file = google_drive3::api::File {
        id: file_id,
        name: Some(file_info.name),
        mime_type: Some(file_info.mime_type.to_string()),
        parents: file_info.parents,
        ..google_drive3::api::File::default()
    };

    let chunk_size = delegate_config.chunk_size;
    let mut delegate = UploadDelegate::new(delegate_config);

    let req = hub
        .files()
        .create(dst_file)
        .param("fields", "id,name,size,createdTime,modifiedTime,md5Checksum,mimeType,parents,shared,description,webContentLink,webViewLink")
        .add_scope(google_drive3::api::Scope::Full)
        .delegate(&mut delegate)
        .supports_all_drives(true);

    let (_, file) = if file_info.size > chunk_size {
        req.upload_resumable(src_file, file_info.mime_type).await?
    } else {
        req.upload(src_file, file_info.mime_type).await?
    };

    Ok(file)
}

#[derive(Debug)]
pub enum Error {
    Hub(hub_helper::Error),
    FileInfo(file_info::Error),
    OpenFile(PathBuf, io::Error),
    Upload(google_drive3::Error),
    IsDirectory(PathBuf),
    DriveFolderMissingId,
    CreateFileTree(file_tree::Error),
    Mkdir(google_drive3::Error),
}

impl error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Hub(err) => write!(f, "{}", err),
            Error::FileInfo(err) => write!(f, "{}", err),
            Error::OpenFile(path, err) => {
                write!(f, "Failed to open file '{}': {}", path.display(), err)
            }
            Error::Upload(err) => write!(f, "Failed to upload file: {}", err),
            Error::IsDirectory(path) => write!(
                f,
                "'{}' is a directory, use --recursive to upload directories",
                path.display()
            ),
            Error::DriveFolderMissingId => write!(f, "Folder created on drive does not have an id"),
            Error::CreateFileTree(err) => write!(f, "Failed to create file tree: {}", err),
            Error::Mkdir(err) => write!(f, "Failed to create directory: {}", err),
        }
    }
}

fn err_if_directory(path: &PathBuf, config: &Config) -> Result<(), Error> {
    if path.is_dir() && !config.upload_directories {
        Err(Error::IsDirectory(path.clone()))
    } else {
        Ok(())
    }
}
