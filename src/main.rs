use crate::utils::sf;
use anyhow::anyhow;
use anyhow::Result;
use clap::{Parser, Subcommand};
use rand::Rng;
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
        #[arg(short = 'o', long = "target-out")]
        target_org: Option<String>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match sf::verify_cli_is_installed() {
        Ok(_) => (),
        Err(x) => return Err(anyhow!(x)),
    }
    let project_config = project_config::read(None);

    match &cli.command {
        Commands::Verify { devhub, target_org } => {
            let scratch_org_name = format!(
                "{}{}",
                project_config.get_name(),
                rand::thread_rng().gen::<usize>()
            );
            println!("scratch name {}", scratch_org_name);
            let command_run =
                commands::verify::run(&scratch_org_name, devhub, target_org, &project_config);

            if target_org.is_none() {
                sf::Cli::new(scratch_org_name.to_owned()).delete_old_scratch()?;
            }

            if command_run.is_err() {
                return Err(anyhow!(command_run.unwrap_err()));
            }

            Ok(())
        }
    }
}
