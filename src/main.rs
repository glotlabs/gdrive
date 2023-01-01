pub mod about;
pub mod account;
pub mod common;
pub mod config;
pub mod files;
pub mod hub;
pub mod version;

use std::{error::Error, path::PathBuf};

use clap::{Parser, Subcommand};
use mime::Mime;

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
        /// account name
        account_name: String,
    },
}

#[derive(Subcommand)]
enum FileCommand {
    /// List files
    List,

    /// Upload files
    Upload {
        /// Path of file to upload
        file_path: PathBuf,

        /// Force mime type (default: auto-detect)
        mime_type: Option<Mime>,
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
            }
        }

        Command::Files { command } => {
            match command {
                FileCommand::List => {
                    // fmt
                    files::list().await.unwrap_or_else(handle_error)
                }

                FileCommand::Upload {
                    file_path,
                    mime_type,
                } => {
                    // fmt
                    files::upload(files::upload::Config {
                        file_path,
                        mime_type,
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
