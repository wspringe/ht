use crate::cli::sf::SalesforceCli;
use crate::project;
use crate::project_config::SalesforceProjectConfig;
use anyhow::Result;

pub fn run(
    scratch_org_name: &String,
    devhub: &Option<String>,
    target_org: &Option<String>,
    project_config: &mut SalesforceProjectConfig,
) -> Result<()> {
    let devhub_alias = match devhub {
        Some(x) => x,
        None => &String::from("DevHub"),
    };

    let mut cli: SalesforceCli;
    if target_org.is_none() {
        cli = SalesforceCli::new(Some(scratch_org_name.to_owned()));
        cli.create_scratch_org(devhub_alias)?;
    } else {
        cli = SalesforceCli::new(Some(target_org.to_owned().unwrap()));
    }

    if let Some(dependencies) = project_config.get_dependencies() {
        for dependency in dependencies {
            cli.install_package(dependency.id.as_str())?;
        }
    }

    project::exec_predeploy_scripts(cli.to_owned())?;
    // deploy metadata
    for package in project_config.get_packages() {
        if let Some(path) = &package.unpackaged_metadata {
            cli.project_deploy(path.as_str())?;
        }
    }
    for package in project_config.get_packages() {
        cli.project_deploy(package.path.as_str())?;
    }

    project::exec_postdeploy_scripts(cli.to_owned())?;

    // run tests
    cli.run_tests()?;

    Ok(())
}
