use anyhow::anyhow;
use anyhow::Result;
use clap::{Parser, Subcommand};
use cli::sf;
use rand::Rng;

mod cli;
mod commands;
mod project;
mod project_config;
mod system;

#[derive(Parser)]
#[clap(name = "HT", about = "Salesforce Build Tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Verify {
        #[arg(short = 'v', long = "devhub")]
        devhub: Option<String>,
        #[arg(short = 'o', long = "target-out")]
        target_org: Option<String>,
    },
    Version {
        #[arg(long = "dry-run")]
        dry_run: bool,
        #[arg(short = 'v', long = "devhub")]
        devhub: Option<String>,
        #[arg(long = "push", help = "Git push after committing")]
        push: bool,
    },
    #[command(about = "Releases the package")]
    Release {},
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match sf::verify_cli_is_installed() {
        Ok(_) => (),
        Err(x) => return Err(anyhow!(x)),
    }
    let mut project_config = project_config::read(None);

    match &cli.command {
        Commands::Verify { devhub, target_org } => {
            let scratch_org_name = format!(
                "{}{}",
                project_config.get_name(),
                rand::thread_rng().gen::<usize>()
            );
            println!("scratch name {}", scratch_org_name);
            let command_run =
                commands::verify::run(&scratch_org_name, devhub, target_org, &mut project_config);

            if target_org.is_none() {
                sf::SalesforceCli::new(Some(scratch_org_name.to_owned())).delete_old_scratch()?;
            }

            if command_run.is_err() {
                return Err(anyhow!(command_run.unwrap_err()));
            }

            Ok(())
        }
        Commands::Version {
            dry_run,
            devhub,
            push,
        } => commands::version::run(&mut project_config, dry_run, push, devhub),
        Commands::Release {} => {
            todo!()
        }
    }
}
