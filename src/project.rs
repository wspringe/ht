use anyhow::Result;
use std::{ffi::OsStr, fs};

use crate::cli::sf::SalesforceCli;

use super::system;

enum ScriptType {
    Shell,
    Apex,
    Unknown,
}

pub struct Script {
    path: String,
    s_type: ScriptType,
}

pub fn exec_predeploy_scripts(cli: SalesforceCli) -> Result<()> {
    match get_predeploy_scripts() {
        Ok(x) => {
            exec_scripts(x, cli);
            Ok(())
        }
        Err(_) => Ok(()),
    }
}

fn exec_scripts(scripts: Vec<Script>, mut cli: SalesforceCli) {
    for script in scripts {
        match script.s_type {
            ScriptType::Apex => {
                cli.exec_anonymous(&script.path);
            }
            ScriptType::Shell => {
                system::exec_script(&script.path);
            }
            ScriptType::Unknown => {
                println!("Script is not an accepted type")
            }
        }
    }
}

pub fn exec_postdeploy_scripts(cli: SalesforceCli) -> Result<()> {
    match get_postdeploy_scripts() {
        Ok(x) => {
            exec_scripts(x, cli);
            Ok(())
        }
        Err(_) => Ok(()),
    }
}

fn get_predeploy_scripts() -> Result<Vec<Script>> {
    let paths = fs::read_dir("deploy/pre")?;
    Ok(get_scripts(paths))
}

fn get_scripts(paths: fs::ReadDir) -> Vec<Script> {
    let mut scripts: Vec<Script> = Vec::new();

    paths.for_each(|entry| {
        if let Ok(entry) = entry {
            let s_type = if get_extension(&entry) == "apex" {
                ScriptType::Apex
            } else if get_extension(&entry) == "sh" {
                ScriptType::Shell
            } else {
                ScriptType::Unknown
            };

            scripts.push(Script {
                path: entry.path().into_os_string().into_string().unwrap(),
                s_type,
            })
        }
    });

    scripts
}

fn get_extension(entry: &fs::DirEntry) -> String {
    entry
        .path()
        .extension()
        .and_then(OsStr::to_str)
        .unwrap()
        .to_owned()
}

pub fn get_postdeploy_scripts() -> Result<Vec<Script>> {
    let paths = fs::read_dir("deploy/post")?;
    Ok(get_scripts(paths))
}
