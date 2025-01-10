use cli_table::print_stdout;

use crate::sf::Cli;
use crate::sf::SfCliResult;
use crate::utils::project;
use crate::utils::project_config::ProjectConfig;
use anyhow::Result;

pub fn run(
    scratch_org_name: &String,
    devhub: &Option<String>,
    target_org: &Option<String>,
    project_config: &ProjectConfig,
) -> Result<()> {
    let devhub_alias = match devhub {
        Some(x) => x,
        None => &String::from("DevHub"),
    };

    let mut cli: Cli;
    if target_org.is_none() {
        cli = Cli::new(scratch_org_name.to_owned());
        print_stdout(
            cli.create_scratch_org(devhub_alias)?
                .get_formatted_results(),
        )
        .expect("the scratch org should be created");
    } else {
        cli = Cli::new(target_org.to_owned().unwrap());
    }

    for package in project_config.get_packages() {
        cli.install_package(&package.id)?;
    }

    // deploy unpackaged metadata if unspecified (should be before or after?)
    if project_config.get_unpackaged_metadata_path().is_some() {
        print_stdout(
            cli.project_deploy(
                &project_config
                    .get_unpackaged_metadata_path()
                    .clone()
                    .unwrap(),
            )?
            .get_formatted_results(),
        )
        .expect("the unpackaged metadata should be deployed");
    }

    project::exec_predeploy_scripts(cli.to_owned())?;
    // deploy metadata
    for path in project_config.get_paths() {
        print_stdout(cli.project_deploy(path)?.get_formatted_results())
            .expect("the project should be deployed");
    }
    project::exec_postdeploy_scripts(cli.to_owned())?;

    // run tests
    cli.run_tests()?;

    Ok(())
}
