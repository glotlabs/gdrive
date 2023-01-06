pub mod about;
pub mod account;
pub mod common;
pub mod config;
pub mod files;
pub mod hub;
pub mod md5_reader;
pub mod version;

use clap::{Parser, Subcommand};
use files::download::ExistingFileAction;
use files::list::ListQuery;
use files::list::ListSortOrder;
use mime::Mime;
use std::{error::Error, path::PathBuf};

#[derive(Parser)]
#[command(author, version, about, long_about = None, disable_version_flag = true)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Print information about gdrive
    About,

    /// Commands for managing accounts
    Account {
        #[command(subcommand)]
        command: AccountCommand,
    },

    /// Commands for managing files
    Files {
        #[command(subcommand)]
        command: FileCommand,
    },

    /// Print version information
    Version,
}

#[derive(Subcommand)]
enum AccountCommand {
    /// Add an account
    Add,

    /// List all accounts
    List,

    /// Print current account
    Current,

    /// Switch to a different account
    Switch {
        /// Account name
        account_name: String,
    },

    /// Remove an account
    Remove {
        /// Account name
        account_name: String,
    },

    /// Export account, this will create a zip file of the account which can be imported
    Export {
        /// Account name
        account_name: String,
    },
}

#[derive(Subcommand)]
enum FileCommand {
    /// Print file info
    Info {
        /// File id
        file_id: String,

        /// Display size in bytes
        #[arg(long, default_value_t = false)]
        size_in_bytes: bool,
    },

    /// List files
    List {
        /// Max files to list
        #[arg(long, default_value_t = 30)]
        max: usize,
    },

    /// Upload files
    Upload {
        /// Path of file to upload
        file_path: PathBuf,

        /// Force mime type [default: auto-detect]
        #[arg(long, value_name = "MIME_TYPE")]
        mime: Option<Mime>,

        /// Upload to an existing folder, multiple parents can be specified
        #[arg(long, value_name = "ID")]
        parent: Option<Vec<String>>,
    },

    /// Download files
    Download {
        /// File id
        file_id: String,

        /// Overwrite existing files
        #[arg(long)]
        overwrite: bool,
    },

    /// Download files
    Delete {
        /// File id
        file_id: String,

        /// Delete directory and all it's content
        #[arg(long)]
        recursive: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::About => {
            // fmt
            about::about()
        }

        Command::Account { command } => {
            // fmt
            match command {
                AccountCommand::Add => {
                    // fmt
                    account::add().await.unwrap_or_else(handle_error)
                }

                AccountCommand::List => {
                    // fmt
                    account::list().unwrap_or_else(handle_error)
                }

                AccountCommand::Current => {
                    // fmt
                    account::current().unwrap_or_else(handle_error)
                }

                AccountCommand::Switch { account_name } => {
                    // fmt
                    account::switch(&account_name).unwrap_or_else(handle_error)
                }

                AccountCommand::Remove { account_name } => {
                    // fmt
                    account::remove(&account_name).unwrap_or_else(handle_error)
                }

                AccountCommand::Export { account_name } => {
                    // fmt
                    account::export(&account_name).unwrap_or_else(handle_error)
                }
            }
        }

        Command::Files { command } => {
            match command {
                FileCommand::Info {
                    file_id,
                    size_in_bytes,
                } => {
                    // fmt
                    files::info(files::info::Config {
                        file_id,
                        size_in_bytes,
                    })
                    .await
                    .unwrap_or_else(handle_error)
                }

                FileCommand::List { max } => {
                    // fmt
                    files::list(files::list::Config {
                        query: ListQuery::default(),
                        order_by: ListSortOrder::default(),
                        max_files: max,
                    })
                    .await
                    .unwrap_or_else(handle_error)
                }

                FileCommand::Upload {
                    file_path,
                    mime,
                    parent,
                } => {
                    // fmt
                    files::upload(files::upload::Config {
                        file_path,
                        mime_type: mime,
                        parents: parent,
                    })
                    .await
                    .unwrap_or_else(handle_error)
                }

                FileCommand::Download { file_id, overwrite } => {
                    let existing_file_action = if overwrite {
                        ExistingFileAction::Overwrite
                    } else {
                        ExistingFileAction::Abort
                    };

                    files::download(files::download::Config {
                        file_id,
                        existing_file_action,
                        download_directories: false,
                    })
                    .await
                    .unwrap_or_else(handle_error)
                }

                FileCommand::Delete { file_id, recursive } => {
                    // fmt
                    files::delete(files::delete::Config {
                        file_id,
                        delete_directories: recursive,
                    })
                    .await
                    .unwrap_or_else(handle_error)
                }
            }
        }

        Command::Version => {
            // fmt
            version::version()
        }
    }
}

fn handle_error(err: impl Error) {
    eprintln!("Error: {}", err);
    std::process::exit(1);
}
