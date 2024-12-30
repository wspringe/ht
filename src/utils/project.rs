use super::{sf, system};
use std::{ffi::OsStr, fs};

enum ScriptType {
    Shell,
    Apex,
    Unknown,
}

struct Script {
    path: String,
    s_type: ScriptType,
}

pub fn exec_predeploy_scripts() {
    let scripts = get_predeploy_scripts();
    exec_scripts(scripts);
}

fn exec_scripts(scripts: Vec<Script>) {
    for script in scripts {
        match script.s_type {
            ScriptType::Apex => {
                sf::exec_anonymous(&script.path);
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

pub fn exec_postdeploy_scripts() {
    let scripts = get_postdeploy_scripts();
    exec_scripts(scripts);
}

fn get_predeploy_scripts() -> Vec<Script> {
    let paths = fs::read_dir("deploy/pre").expect("Did not find a deploy/pre directory");
    get_scripts(paths)
}

fn get_scripts(paths: fs::ReadDir) -> Vec<Script> {
    let mut scripts: Vec<Script> = Vec::new();

    for entry in paths {
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
    }

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

pub fn get_postdeploy_scripts() -> Vec<Script> {
    let paths = fs::read_dir("deploy/post").expect("Did not find a deploy/post directory");
    get_scripts(paths)
}
