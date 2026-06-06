use clap::{Parser, Subcommand};

mod config;
mod gist;
mod history;
mod pull;
mod push;
mod restore;
mod util;

#[derive(Parser)]
#[command(name = "zed-config", version, about = "Zed editor config sync tool")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Show the current GitHub token
    Token,
    /// Show the current Gist ID
    Gist,
    /// Set a configuration value
    Set {
        #[command(subcommand)]
        field: SetField,
    },
    /// Pull latest config from cloud to local
    Pull,
    /// Push local config to cloud
    Push,
    /// List history records in the current gist
    History,
    /// Restore a specific config by date ID (format: YYYY-MM-DD_HH:mm)
    Restore {
        /// Date ID of the config snapshot to restore
        config_id: String,
    },
    /// Show the current version
    Version,
}

#[derive(Subcommand)]
enum SetField {
    /// Set the GitHub personal access token
    Token {
        /// The token value
        token: String,
    },
    /// Set the Gist ID
    Gist {
        /// The Gist ID value
        gist_id: String,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {:#}", e);
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    match Cli::parse().command {
        Command::Token => {
            let config = config::Config::load_or_default();
            if config.github_token.is_empty() {
                println!("(not set)");
            } else {
                println!("{}", config.github_token);
            }
        }
        Command::Gist => {
            let config = config::Config::load_or_default();
            if config.gist_id.is_empty() {
                println!("(not set)");
            } else {
                println!("{}", config.gist_id);
            }
        }
        Command::Set { field } => match field {
            SetField::Token { token } => {
                print!("Validating token... ");
                if !gist::validate_token(&token)? {
                    eprintln!("invalid.");
                    anyhow::bail!("The provided token is not valid.");
                }
                println!("valid.");
                let mut config = config::Config::load_or_default();
                config.github_token = token;
                config.save()?;
                println!("Token saved.");
            }
            SetField::Gist { gist_id } => {
                let config = config::Config::load_or_default();
                if config.github_token.is_empty() {
                    anyhow::bail!(
                        "No token configured. Please set a token first with: zed-config set token <TOKEN>"
                    );
                }
                print!("Validating token... ");
                if !gist::validate_token(&config.github_token)? {
                    eprintln!("invalid.");
                    anyhow::bail!(
                        "Current token is invalid. Please update it with: zed-config set token <TOKEN>"
                    );
                }
                println!("valid.");
                print!("Validating gist ID... ");
                if !gist::validate_gist(&config.github_token, &gist_id)? {
                    eprintln!("not found.");
                    anyhow::bail!("Gist ID does not exist or is not accessible.");
                }
                println!("valid.");
                let mut config = config;
                config.gist_id = gist_id;
                config.save()?;
                println!("Gist ID saved.");
            }
        },
        Command::Pull => pull::run()?,
        Command::Push => push::run()?,
        Command::History => history::run()?,
        Command::Restore { config_id } => restore::run(&config_id)?,
        Command::Version => println!("{}", env!("CARGO_PKG_VERSION")),
    }
    Ok(())
}
