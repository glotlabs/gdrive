pub mod config;
pub mod hub;

use clap::{Parser, Subcommand};
use std::io::{self, Write};

use crate::config::Config;

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
                    if let Err(err) = add_account().await {
                        eprintln!("Error: {:?}", err);
                    }
                }
            }
        }

        Command::List => {
            // fmt
            if let Err(err) = list().await {
                eprintln!("Error: {:?}", err);
            }
        }
    }

    ()
}

async fn add_account() -> Result<(), Error> {
    let secret = secret_prompt()?;

    let tmp_dir = tempfile::tempdir().map_err(Error::Tempdir)?;
    let tokens_path = tmp_dir.path().join("tokens.json");

    let auth = hub::Auth::new(&secret, &tokens_path)
        .await
        .map_err(Error::Auth)?;

    // Get access tokens
    auth.token(&[
        "https://www.googleapis.com/auth/drive",
        "https://www.googleapis.com/auth/drive.metadata.readonly",
    ])
    .await
    .map_err(Error::AccessToken)?;

    let hub = hub::Hub::new(auth).await;
    let (_, about) = hub
        .about()
        .get()
        .param("fields", "user")
        .doit()
        .await
        .map_err(Error::About)?;

    let email = about
        .user
        .and_then(|u| u.email_address)
        .unwrap_or_else(|| String::from("unknown"));

    let config = config::add_account(&email, &secret, &tokens_path).map_err(Error::Config)?;
    config::switch_account(&config).map_err(Error::Config)?;

    println!("Logged in as {}", config.account.name);

    Ok(())
}

async fn list() -> Result<(), Error> {
    let config = Config::load_current_account().map_err(Error::Config)?;
    let secret = config.load_secret().map_err(Error::Config)?;
    let auth = hub::Auth::new(&secret, &config.tokens_path())
        .await
        .map_err(Error::Auth)?;

    let hub = hub::Hub::new(auth).await;
    let res = hub.files().list().doit().await;
    println!("{:?}", res);
    Ok(())
}

fn secret_prompt() -> Result<config::Secret, Error> {
    println!("A client id and client secret are required to use this application.");
    println!();

    let client_id = prompt_input("Client ID")?;
    let client_secret = prompt_input("Client secret")?;

    Ok(config::Secret {
        client_id,
        client_secret,
    })
}

fn prompt_input(msg: &str) -> Result<String, Error> {
    print!("{}: ", msg);
    let _ = io::stdout().flush();

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(Error::ReadStdin)?;

    Ok(input.trim().to_string())
}

#[derive(Debug)]
enum Error {
    Config(config::Error),
    Tempdir(io::Error),
    Auth(io::Error),
    ReadStdin(io::Error),
    AccessToken(google_drive3::oauth2::Error),
    About(google_drive3::Error),
}
