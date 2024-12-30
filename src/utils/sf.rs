use anyhow::anyhow;
use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize)]
struct SfCliCommandOutput {
    name: String,
    message: Option<String>,
    result: Option<String>,
    status: u8,
}

use std::{
    fmt::{self, Display, Formatter},
    process::Command,
};

#[derive(Debug)]
pub struct SfCliError;

impl Display for SfCliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "something went wrong using the SF CLI")
    }
}

pub fn verify_cli_is_installed() -> Result<()> {
    match Command::new("sf").spawn() {
        Ok(_) => Ok(()),
        Err(..) => Err(anyhow!("SF CLI not found")),
    }
}

pub fn create_scratch_org(devhub: &String, alias: &String) -> Result<SfCliCommandOutput> {
    let command = Command::new("sf")
        .args([
            "org",
            "create",
            "scratch",
            "-v",
            devhub,
            "--definition-file",
            "config/project-scratch-def.json",
            "--alias",
            alias,
        ])
        .output();

    match command {
        Ok(x) => {
            let output = String::from_utf8(x.stdout).unwrap();
            let command_output: SfCliCommandOutput = serde_json::from_str(output.as_str())
                .expect("could not deserialize sf cli command output");
            if command_output.name.contains("Error") {
                return Err(anyhow!(SfCliError).context(format!(
                    "could not create scratch org: {}",
                    command_output.message.unwrap(),
                )));
            }
            Ok(command_output)
        }
        Err(e) => Err(anyhow!(SfCliError).context(e.to_string())),
    }
}

pub fn delete_old_scratch(scratch_name: &String) -> Result<SfCliCommandOutput> {
    let command = Command::new("sf")
        .args(["org", "delete", "scratch", "--target-org", scratch_name])
        .output();

    match command {
        Ok(x) => {
            let output = String::from_utf8(x.stdout).unwrap();
            let command_output: SfCliCommandOutput = serde_json::from_str(output.as_str())
                .expect("could not deserialize sf cli command output");
            if command_output.name.contains("Error") {
                return Err(anyhow!(SfCliError).context(format!(
                    "could not delete scratch org: {}",
                    command_output.message.unwrap(),
                )));
            }
            Ok(command_output)
        }
        Err(e) => Err(anyhow!(SfCliError).context(e.to_string())),
    }
}

pub fn auth_devhub(path_to_auth_file: &String) -> Result<SfCliCommandOutput> {
    let command = Command::new("sf")
        .args([
            "org",
            "login",
            "sfdx-url",
            "--sfdx-url-file",
            path_to_auth_file,
        ])
        .output();

    match command {
        Ok(x) => {
            let output = String::from_utf8(x.stdout).unwrap();
            let command_output: SfCliCommandOutput = serde_json::from_str(output.as_str())
                .expect("could not deserialize sf cli command output");
            if command_output.name.contains("Error") {
                return Err(anyhow!(SfCliError).context(format!(
                    "could not authorize devhub: {}",
                    command_output.message.unwrap(),
                )));
            }
            Ok(command_output)
        }
        Err(e) => Err(anyhow!(SfCliError).context(e.to_string())),
    }
}

pub fn project_deploy(path: &String) -> Result<SfCliCommandOutput> {
    let command = Command::new("sf")
        .args(["project", "deploy", "start", "-d", path])
        .output();

    match command {
        Ok(x) => {
            let output = String::from_utf8(x.stdout).unwrap();
            let command_output: SfCliCommandOutput = serde_json::from_str(output.as_str())
                .expect("could not deserialize sf cli command output");
            if command_output.name.contains("Error") {
                return Err(anyhow!(SfCliError).context(format!(
                    "could not create scratch org: {}",
                    command_output.message.unwrap(),
                )));
            }
            Ok(command_output)
        }
        Err(e) => Err(anyhow!(SfCliError).context(e.to_string())),
    }
}

pub fn exec_anonymous(path: &String) -> Result<SfCliCommandOutput> {
    let command = Command::new("sf")
        .args(["apex", "run", "--file", path])
        .output();

    match command {
        Ok(x) => {
            let output = String::from_utf8(x.stdout).unwrap();
            let command_output: SfCliCommandOutput = serde_json::from_str(output.as_str())
                .expect("could not deserialize sf cli command output");
            if command_output.name.contains("Error") {
                return Err(anyhow!(SfCliError).context(format!(
                    "could not execute anonymous apex: {}",
                    command_output.message.unwrap(),
                )));
            }
            Ok(command_output)
        }
        Err(e) => Err(anyhow!(SfCliError).context(e.to_string())),
    }
}

pub fn run_tests() -> Result<SfCliCommandOutput> {
    let command = Command::new("sf")
        .args(["apex", "test", "run", "-c", "-w", "60"])
        .output();

    match command {
        Ok(x) => {
            let output = String::from_utf8(x.stdout).unwrap();
            let command_output: SfCliCommandOutput = serde_json::from_str(output.as_str())
                .expect("could not deserialize sf cli comman output");
            if command_output.name.contains("Error") {
                return Err(anyhow!(SfCliError).context(format!(
                    "could not execute apex tests: {}",
                    command_output.message.unwrap(),
                )));
            }
            Ok(command_output)
        }
        Err(e) => Err(anyhow!(SfCliError).context(e.to_string())),
    }
}
