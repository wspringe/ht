// TODO: refactor into separate modules for each command (makes it easier to handle cli command outputs)

use anyhow::anyhow;
use anyhow::Result;
use cli_table::format::Justify;
use cli_table::{Cell, Style, Table, TableStruct};
use serde::Deserialize;
use std::ops::Deref;
use std::{
    fmt::{self, Display, Formatter},
    process::Command,
};

trait SfCliResult {
    fn get_formatted_results(&self) -> TableStruct;
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TestResult {
    message: String,
    method_name: String,
    name: String,
    stack_trace: String,
    time: u32,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RunTestResult {
    failures: Vec<TestResult>,
    successes: Vec<TestResult>,
    num_failures: u32,
    num_tests_run: u32,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct MetadataComponent {
    component_type: String,
    full_name: String,
    problem: String,
    success: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeployDetails {
    component_successes: Vec<MetadataComponent>,
    component_failures: Vec<MetadataComponent>,
    run_test_result: RunTestResult,
}

#[derive(Deserialize)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
enum CliResult {
    CreateScratchOrgResult {
        username: String,
        id: String,
        features: String,
    },
    DeleteScratchOrgResult {
        username: String,
        org_id: String,
    },
    AuthorizeResult {
        username: String,
    },
    ProjectDeployResult {
        details: DeployDetails,
    },
    ExecuteAnonymousApexResult {
        success: bool,
        compile_problem: String,
        exception_message: String,
        exception_stack_trace: String,
    },
}
impl SfCliResult for CliResult {
    fn get_formatted_results(&self) -> TableStruct {
        match self {
            CliResult::CreateScratchOrgResult {
                username,
                id,
                features,
            } => vec![
                vec!["Id".cell(), id.cell().justify(Justify::Right)],
                vec!["Username".cell(), username.cell().justify(Justify::Right)],
            ]
            .table()
            .title(vec!["Scratch Org".cell().bold(true)])
            .bold(true),
            _ => todo!(),
        }
    }
}

#[derive(Deserialize)]
pub struct SfCliCommandOutput {
    name: String,
    message: Option<String>,
    result: Option<CliResult>,
    status: u8,
}

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
pub struct Cli {
    command: Command,
    output: String,
}
impl Cli {
    pub(crate) fn new() -> Self {
        Cli {
            command: Command::new("sf"),
            output: String::new(),
        }
    }

    fn mock_cli_output(&mut self, output: String) -> &mut Self {
        self.output = output;
        self
    }

    pub fn create_scratch_org(
        &mut self,
        devhub: &String,
        alias: &String,
    ) -> Result<SfCliCommandOutput> {
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
                let output = if self.output.is_empty() {
                    String::from_utf8(x.stdout)?
                } else {
                    self.output.clone()
                };
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

    pub(crate) fn exec_anonymous(path: &String) -> Result<SfCliCommandOutput> {
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
}
