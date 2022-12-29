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
    /// auth commands
    Auth {
        #[command(subcommand)]
        command: AuthCommand,
    },

    /// list commands
    List,
}

#[derive(Subcommand)]
enum AuthCommand {
    /// login
    Login,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    match cli.command {
        Command::Auth { command } => {
            // fmt
            match command {
                AuthCommand::Login => {
                    // fmt
                    if let Err(err) = login().await {
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

async fn login() -> Result<(), Error> {
    let config = Config::init("1").map_err(Error::Config)?;
    let secret = get_secret(&config)?;
    let auth = hub::Auth::new(secret, &config.tokens_path())
        .await
        .map_err(Error::Auth)?;

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

    println!("Logged in as {}", email);

    Ok(())
}

async fn list() -> Result<(), Error> {
    let config = Config::init("1").map_err(Error::Config)?;
    let secret = get_secret(&config)?;
    let auth = hub::Auth::new(secret, &config.tokens_path())
        .await
        .map_err(Error::Auth)?;

    let hub = hub::Hub::new(auth).await;
    let res = hub.files().list().doit().await;
    println!("{:?}", res);
    Ok(())
}

fn get_secret(config: &Config) -> Result<config::Secret, Error> {
    if let Ok(secret) = config.load_secret() {
        Ok(secret)
    } else {
        let secret = secret_prompt()?;
        let _ = config.save_secret(&secret);
        Ok(secret)
    }
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
    Auth(io::Error),
    ReadStdin(io::Error),
    AccessToken(google_drive3::oauth2::Error),
    About(google_drive3::Error),
}
