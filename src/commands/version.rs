use std::{collections::HashMap, fs, io::Write};

use anyhow::Result;
use git2::{IndexAddOption, Repository, Signature};
use indexmap::IndexMap;
use serde_json::{json, Value};

use crate::{
    cli::sf::SalesforceCli,
    project_config::{Package, SalesforceProjectConfig, Version},
};

pub fn run(project_config: &mut SalesforceProjectConfig, dry_run: &bool) -> Result<()> {
    // figure out which package you want
    let repo = Repository::open(".").unwrap();
    let head = repo.head().unwrap();
    let oid = head.target().unwrap();
    let commit = repo.find_commit(oid).unwrap();
    dbg!(&commit);
    let message = commit.message().unwrap();
    dbg!(&message);

    let split = &message.split(':').collect::<Vec<&str>>();
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
    let project_json_path = String::from("./sfdx-project.json");
    let mut file = fs::read_to_string(project_json_path).expect("Did not find sfdx-project.json");
    let mut config: IndexMap<String, Value> =
        serde_json::from_str(&file).expect("unable to parse json");
    dbg!(&config);
    for package_dir in config
        .get_mut("packageDirectories")
        .unwrap()
        .as_array_mut()
        .unwrap()
    {
        if package_dir.get_mut("package").unwrap().as_str().unwrap() == to_upgrade.name {
            let version_number = package_dir.get_mut("versionNumber").unwrap();
            *version_number = json!(new_version.to_string());
        }
    }

    let json_string = serde_json::to_string_pretty(&config)?;
    dbg!(&json_string);
    let mut f = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open("./sfdx-project.json")
        .expect("should have opened the sfdx project file");
    f.write_all(&json_string.into_bytes())
        .expect("should have overwrote the sfdx project json file");
    f.flush()?;

    // create new version of unlocked package
    let mut cli = SalesforceCli::new(None);
    if !dry_run {
        cli.create_package_version("")?;
    }
    // create new commmit and tag it with new version
    // https://users.rust-lang.org/t/using-git2-to-clone-create-a-branch-and-push-a-branch-to-github/100292
    create_commit(&repo);
    Ok(())
}

fn create_commit(repo: &Repository) {
    // stage changes
    let mut index = repo.index().unwrap();
    index.add_all(&["."], IndexAddOption::DEFAULT, None);
    index.write().unwrap();

    let signature = repo.signature().unwrap();
    let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();
    let parent_commit = repo.head().unwrap().peel_to_commit().unwrap();

    let commit_oid = repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "ci: making new version",
        &tree,
        &[&parent_commit],
    );
    dbg!(&commit_oid);
}
