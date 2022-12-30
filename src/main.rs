pub mod account;
pub mod config;
pub mod gdrive;
pub mod hub;

use std::error::Error;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// account commands
    Account {
        #[command(subcommand)]
        command: AccountCommand,
    },

    /// list commands
    List,
}

#[derive(Subcommand)]
enum AccountCommand {
    /// add
    Add,

    /// list
    List,

    /// current
    Current,

    /// switch
    Switch {
        /// account name
        account_name: String,
    },

    /// remove
    Remove {
        /// account name
        account_name: String,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
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

        Command::List => {
            // fmt
            gdrive::list().await.unwrap_or_else(handle_error)
        }
    }
}

fn handle_error(err: impl Error) {
    eprintln!("Error: {}", err);
    std::process::exit(1);
}
