use cli_table::print_stdout;

use crate::sf::Cli;
use crate::sf::SfCliCommandOutput;
use crate::sf::SfCliResult;
use crate::utils::project;
use crate::utils::project_config::ProjectConfig;
use anyhow::Result;

pub fn run(
    scratch_org_name: &String,
    devhub: &Option<String>,
    project_config: &ProjectConfig,
) -> Result<()> {
    let devhub_alias = match devhub {
        Some(x) => x,
        None => &String::from("DevHub"),
    };

    let mut cli_results: Vec<SfCliCommandOutput> = Vec::new();
    let mut cli = Cli::new();
    println!("creating scratch");
    cli_results.push(cli.create_scratch_org(devhub_alias, scratch_org_name)?);

    println!("installing any dependencies");
    for package in project_config.get_packages() {
        cli.install_package(&package.id)?;
    }

    println!("deploying unpackaged metadata");
    // deploy unpackaged metadata if unspecified (should be before or after?)
    if project_config.get_unpackaged_metadata_path().is_some() {
        cli_results.push(
            cli.project_deploy(
                &project_config
                    .get_unpackaged_metadata_path()
                    .clone()
                    .unwrap(),
            )?,
        );
    }

    println!("exec pre");
    project::exec_predeploy_scripts()?;
    // deploy metadata
    println!("deploying");
    for path in project_config.get_paths() {
        cli_results.push(cli.project_deploy(path)?);
    }
    println!("post");
    project::exec_postdeploy_scripts()?;

    // run tests
    println!("running tests");
    cli_results.push(cli.run_tests()?);
    // display results
    // TODO: implement a display for everything things that happened this run (scipts run, test results, paths deployed;  option for more verbose/debug results like every metadata deployed)
    for cli_result in cli_results {
        let _ = print_stdout(cli_result.get_formatted_results());
    }
    Ok(())
}
