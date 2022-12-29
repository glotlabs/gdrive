pub mod account;
pub mod config;
pub mod gdrive;
pub mod hub;

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
                    if let Err(err) = account::add::add().await {
                        eprintln!("Error: {:?}", err);
                    }
                }
            }
        }

        Command::List => {
            // fmt
            if let Err(err) = gdrive::list::list().await {
                eprintln!("Error: {:?}", err);
            }
        }
    }

    ()
}
