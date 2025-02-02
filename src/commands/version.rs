use anyhow::{anyhow, Result};
use git2::{IndexAddOption, Repository};
use indexmap::IndexMap;
use serde_json::{json, Value};
use std::{fs, io::Write};

use crate::{
    cli::sf::SalesforceCli,
    project_config::{Package, SalesforceProjectConfig, Version},
};

pub fn run(
    project_config: &mut SalesforceProjectConfig,
    dry_run: &bool,
    push: &bool,
    devhub: &Option<String>,
) -> Result<()> {
    if !dry_run && devhub.is_none() {
        return Err(anyhow!("devhub is required"));
    }

    let repo = Repository::open(".").unwrap();
    let message = get_latest_commit_message(&repo);

    let commit_message_split = &message.split(':').collect::<Vec<&str>>();
    let commit_prefix = commit_message_split[0];

    let package_name = get_package_name_from_commit(commit_prefix);
    let to_upgrade: &mut Package = match project_config.get_package(&package_name) {
        Ok(package) => package,
        Err(_) => project_config.get_default_package()?,
    };

    let commit_prefix = commit_message_split[0].split('(').collect::<Vec<&str>>()[0];
    let current_version = Version::from(&to_upgrade.version_number);
    let mut new_version = current_version;
    bump_version(commit_prefix, &mut new_version);
    to_upgrade.set_version(&new_version);

    if new_version.is_higher_than(&current_version) {
        let json_string = generate_new_sfdx_project(to_upgrade, new_version)?;
        write_to_file(json_string)?;

        if !dry_run {
            let mut cli = SalesforceCli::new(None);
            cli.create_package_version(devhub.as_ref().unwrap())?;
        }

        create_commit(&repo)?;
        tag_commit(&repo, &new_version)?;

        if *push {
            let mut origin = repo.find_remote("origin")?;
            origin.push(&[String::new()], None);
        }
    }
    Ok(())
}

fn write_to_file(json_string: String) -> Result<(), anyhow::Error> {
    let mut f = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open("./sfdx-project.json")
        .expect("should have opened the sfdx project file");
    f.write_all(&json_string.into_bytes())
        .expect("should have overwrote the sfdx project json file");
    f.flush()?;
    Ok(())
}

fn generate_new_sfdx_project(
    to_upgrade: &mut Package,
    new_version: Version,
) -> Result<String, anyhow::Error> {
    let project_json_path = String::from("./sfdx-project.json");
    let file_as_string = fs::read_to_string(project_json_path)?;
    let mut config: IndexMap<String, Value> = serde_json::from_str(&file_as_string)?;
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
    Ok(json_string)
}

fn bump_version(commit_prefix: &str, new_version: &mut Version) {
    if commit_prefix.contains("!") {
        new_version.major += 1;
    } else if commit_prefix.contains("feat") {
        new_version.minor += 1;
    } else if commit_prefix.contains("fix") {
        new_version.patch += 1;
    }
}

fn get_package_name_from_commit(commit_prefix: &str) -> String {
    let package_name = if commit_prefix.contains('(') {
        let start_bytes = commit_prefix.find('(').unwrap() + 1;
        let end_bytes = commit_prefix.find(')').unwrap() - 1;
        &commit_prefix[start_bytes..end_bytes]
    } else {
        ""
    };
    package_name.to_string()
}

fn get_latest_commit_message(repo: &Repository) -> String {
    let head = repo.head().unwrap();
    let oid = head.target().unwrap();
    let commit = repo.find_commit(oid).unwrap();
    commit.message().unwrap().to_string()
}

fn create_commit(repo: &Repository) -> Result<()> {
    // stage changes
    let mut index = repo.index().unwrap();
    index.add_all(["."], IndexAddOption::DEFAULT, None)?;
    index.write()?;

    let signature = repo.signature()?;
    let tree = repo.find_tree(index.write_tree()?)?;
    let parent_commit = repo.head().unwrap().peel_to_commit()?;

    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "ci: making new version",
        &tree,
        &[&parent_commit],
    )?;
    Ok(())
}

fn tag_commit(repo: &Repository, version: &Version) -> Result<()> {
    let sig = repo.signature()?;
    let obj = repo.revparse_single("HEAD")?;
    repo.tag(
        &version.to_string(),
        &obj,
        &sig,
        &version.to_string(),
        false,
    )?;

    Ok(())
}
