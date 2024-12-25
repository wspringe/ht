use anyhow::Result;
use clap::{Parser, Subcommand};
use utils::project_config;

mod commands;
mod utils;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Parser)]
#[command(about = "Salesforce Build Tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Verify {
        #[arg(short = 'v', long = "dev-hub")]
        devhub: Option<String>,
        #[arg(long = "delete-old", default_value_t = false)]
        delete_old: bool,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    utils::sf::verify_cli_is_installed();
    let project_config = project_config::read(None);

    match &cli.command {
        Commands::Verify { devhub, delete_old } => {
            println!("Verify was used ");
            commands::verify::run(devhub, delete_old, &project_config);
        }
    }

    Ok(())
}
