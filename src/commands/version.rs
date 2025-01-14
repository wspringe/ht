use anyhow::Result;
use git2::Repository;

use crate::project_config::{SalesforceProjectConfig, Version};

pub fn run(project_config: &SalesforceProjectConfig) -> Result<()> {
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

    let to_upgrade = match project_config.get_package(package_name) {
        Ok(package) => package,
        Err(_) => project_config.get_default_package(),
    };

    let mut version = Version::from(&to_upgrade.version_number);
    let split_again = ["feat:", "blah"];

    if split_again[0].contains("!") {
        version.major += 1;
    } else if split_again[0].contains("feat") {
        version.minor += 1;
    } else {
        version.patch += 1;
    }

    // get current version
    // generate next version based on conventional commit
    // create new version of unlocked package
    // create new commmit and tag it with new version
    todo!()
}
