use anyhow::Result;
use git2::Repository;

use crate::{
    cli::sf::SalesforceCli,
    project_config::{Package, SalesforceProjectConfig, Version},
};

pub fn run(project_config: &mut SalesforceProjectConfig) -> Result<()> {
    // figure out which package you want
    let repo = Repository::open(".").unwrap();
    let head = repo.message().unwrap(); // if this doesn't work, need to revwalk

    let split = &head.split(':').collect::<Vec<&str>>();
    let package_name = if split[0].contains('(') {
        let start_bytes = split[0].find('(').unwrap() + 1;
        let end_bytes = split[0].find(')').unwrap() - 1;
        &split[0][start_bytes..end_bytes]
    } else {
        ""
    };

    let to_upgrade: &mut Package = match project_config.get_package(package_name) {
        Ok(package) => package,
        Err(_) => project_config.get_default_package()?,
    };

    // get commit prefix and bump version
    let commit_prefix = split[0].split('(').collect::<Vec<&str>>()[0];
    let current_version = Version::from(&to_upgrade.version_number);
    let mut new_version = current_version.clone();

    if commit_prefix.contains("!") {
        new_version.major += 1;
    } else if commit_prefix.contains("feat") {
        new_version.minor += 1;
    } else if commit_prefix.contains("fix") {
        new_version.patch += 1;
    }
    to_upgrade.set_version(&new_version);

    // update json
    // create new version of unlocked package
    let cli = SalesforceCli::new(None);
    // cli.create_package_version(devhub);
    // create new commmit and tag it with new version
    todo!()
}
