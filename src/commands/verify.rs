use crate::cli::sf::SalesforceCli;
use crate::project;
use crate::project_config::SalesforceProjectConfig;
use anyhow::Result;

pub fn run(
    scratch_org_name: &String,
    devhub: &Option<String>,
    target_org: &Option<String>,
    project_config: &SalesforceProjectConfig,
) -> Result<()> {
    let devhub_alias = match devhub {
        Some(x) => x,
        None => &String::from("DevHub"),
    };

    let mut cli: SalesforceCli;
    if target_org.is_none() {
        cli = SalesforceCli::new(scratch_org_name.to_owned());
        cli.create_scratch_org(devhub_alias)?;
    } else {
        cli = SalesforceCli::new(target_org.to_owned().unwrap());
    }

    for package in project_config.get_packages() {
        for dependency in package.dependencies.unwrap() {
            cli.install_package(&dependency.id);
        }
    }

    for package in project_config.get_dependencies() {}

    project::exec_predeploy_scripts(cli.to_owned())?;
    // deploy metadata
    for path in project_config.get_paths() {}

    project::exec_postdeploy_scripts(cli.to_owned())?;

    // run tests
    cli.run_tests()?;

    Ok(())
}
