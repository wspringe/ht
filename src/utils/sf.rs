// TODO: refactor into separate modules for each command (makes it easier to handle cli command outputs)

use anyhow::anyhow;
use anyhow::Result;
use cli_table::format::Justify;
use cli_table::{Cell, Style, Table, TableStruct};
use serde::Deserialize;
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
struct ScratchOrgInfo {
    #[serde(rename = "Id")]
    id: String,
    #[serde(rename = "Features")]
    features: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum CliResult {
    CreateScratchOrgResult {
        username: String,
        #[serde(rename = "scratchOrgInfo")]
        scratch_org_info: ScratchOrgInfo,
        #[serde(rename = "orgId")]
        org_id: String,
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
impl SfCliResult for SfCliCommandOutput {
    fn get_formatted_results(&self) -> TableStruct {
        match self.result.as_ref().unwrap() {
            CliResult::CreateScratchOrgResult {
                username,
                scratch_org_info,
                ..
            } => vec![
                vec![
                    "Id".cell(),
                    scratch_org_info.id.clone().cell().justify(Justify::Right),
                ],
                vec!["Username".cell(), username.cell().justify(Justify::Right)],
                vec![
                    "Features".cell(),
                    scratch_org_info
                        .features
                        .replace(';', ", ")
                        .cell()
                        .justify(Justify::Right),
                ],
            ]
            .table()
            .title(vec![
                "Create Scratch Org Results".cell().bold(true),
                "".cell(),
            ])
            .bold(true),
            CliResult::DeleteScratchOrgResult { username, org_id } => vec![
                vec![
                    "Org Id".cell(),
                    org_id.clone().cell().justify(Justify::Right),
                ],
                vec!["Username".cell(), username.cell().justify(Justify::Right)],
                vec![
                    "Is Deleted".cell(),
                    (self.status == 0).cell().justify(Justify::Right),
                ],
            ]
            .table()
            .title(vec![
                "Delete Scratch Org Results".cell().bold(true),
                "".cell(),
            ])
            .bold(true),
            CliResult::AuthorizeResult { .. } => vec![vec![
                "Is Authorized".cell(),
                (self.status == 0).cell().justify(Justify::Right),
            ]]
            .table()
            .title(vec![
                "Authorize Dev Hub Results".cell().bold(true),
                "".cell(),
            ])
            .bold(true),
            CliResult::ProjectDeployResult { details } => vec![
                vec![
                    "Is Successful".cell(),
                    (details.component_failures.len() != 0)
                        .cell()
                        .justify(Justify::Right),
                ],
                vec![
                    "Problems".cell(),
                    details
                        .component_failures
                        .iter()
                        .map(|x| x.problem.clone())
                        .collect::<Vec<_>>()
                        .join("\n")
                        .cell()
                        .justify(Justify::Right),
                ],
                vec![
                    "Is Deleted".cell(),
                    (self.status == 0).cell().justify(Justify::Right),
                ],
            ]
            .table()
            .title(vec![
                "Delete Scratch Org Results".cell().bold(true),
                "".cell(),
            ])
            .bold(true),
            CliResult::AuthorizeResult { .. } => vec![vec![
                "Is Authorized".cell(),
                (self.status == 0).cell().justify(Justify::Right),
            ]]
            .table()
            .title(vec![
                "Authorize Dev Hub Results".cell().bold(true),
                "".cell(),
            ])
            .bold(true),
            _ => vec![vec!["Id".cell(), "id".cell().justify(Justify::Right)]]
                .table()
                .title(vec!["Scratch Org".cell().bold(true)])
                .bold(true),
        }
    }
}

#[derive(Deserialize)]
pub struct SfCliCommandOutput {
    name: Option<String>,
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
        let output = if self.output.is_empty() {
            self.get_output(vec![
                "org",
                "create",
                "scratch",
                "-v",
                devhub,
                "--definition-file",
                "config/project-scratch-def.json",
                "--alias",
                alias,
                "--json",
            ])?
        } else {
            self.output.clone()
        };

        let command_output: SfCliCommandOutput = serde_json::from_str(output.as_str())
            .expect("could not deserialize sf cli command output");
        if command_output.status > 0 {
            return Err(anyhow!(SfCliError).context(format!(
                "could not create scratch org: {}",
                command_output.message.unwrap(),
            )));
        }
        Ok(command_output)
    }

    fn get_output(&mut self, command_args: Vec<&str>) -> Result<String> {
        let command_output = self.command.args(command_args).output();

        match command_output {
            Ok(x) => Ok(String::from_utf8(x.stdout)?),
            Err(e) => return Err(anyhow!(SfCliError).context(e.to_string())),
        }
    }

    pub fn delete_old_scratch(&mut self, scratch_name: &String) -> Result<SfCliCommandOutput> {
        let output = if self.output.is_empty() {
            self.get_output(vec![
                "org",
                "delete",
                "scratch",
                "--target-org",
                scratch_name,
                "--json",
            ])?
        } else {
            self.output.clone()
        };

        let command_output: SfCliCommandOutput = serde_json::from_str(output.as_str())
            .expect("could not deserialize sf cli command output");
        if command_output.status > 0 {
            return Err(anyhow!(SfCliError).context(format!(
                "could not delete scratch org: {}",
                command_output.message.unwrap(),
            )));
        }
        Ok(command_output)
    }

    pub fn auth_devhub(&mut self, path_to_auth_file: &String) -> Result<SfCliCommandOutput> {
        let output = if self.output.is_empty() {
            self.get_output(vec![
                "org",
                "login",
                "sfdx-url",
                "--sfdx-url-file",
                path_to_auth_file,
                "--json",
            ])?
        } else {
            self.output.clone()
        };

        let command_output: SfCliCommandOutput = serde_json::from_str(output.as_str())
            .expect("could not deserialize sf cli command output");
        if command_output.status > 0 {
            return Err(anyhow!(SfCliError).context(format!(
                "could not authorize devhub: {}",
                command_output.message.unwrap(),
            )));
        }
        Ok(command_output)
    }

    pub fn project_deploy(&mut self, path: &String) -> Result<SfCliCommandOutput> {
        let output = if self.output.is_empty() {
            self.get_output(vec!["project", "deploy", "start", "-d", path, "--json"])?
        } else {
            self.output.clone()
        };

        let command_output: SfCliCommandOutput = serde_json::from_str(output.as_str())
            .expect("could not deserialize sf cli command output");
        if command_output.status > 0 {
            return Err(anyhow!(SfCliError).context(format!(
                "could not authorize devhub: {}",
                command_output.message.unwrap(),
            )));
        }
        Ok(command_output)
    }

    pub fn exec_anonymous(&mut self, path: &String) -> Result<SfCliCommandOutput> {
        let output = if self.output.is_empty() {
            self.get_output(vec!["apex", "run", "--file", path, "--json"])?
        } else {
            self.output.clone()
        };

        let command_output: SfCliCommandOutput = serde_json::from_str(output.as_str())
            .expect("could not deserialize sf cli command output");
        if command_output.status > 0 {
            return Err(anyhow!(SfCliError).context(format!(
                "could not authorize devhub: {}",
                command_output.message.unwrap(),
            )));
        }
        Ok(command_output)
    }

    pub fn run_tests(&mut self) -> Result<SfCliCommandOutput> {
        let output = if self.output.is_empty() {
            self.get_output(vec!["apex", "test", "run", "-c", "-w", "60", "--json"])?
        } else {
            self.output.clone()
        };

        let command_output: SfCliCommandOutput = serde_json::from_str(output.as_str())
            .expect("could not deserialize sf cli command output");
        if command_output.status > 0 {
            return Err(anyhow!(SfCliError).context(format!(
                "could not authorize devhub: {}",
                command_output.message.unwrap(),
            )));
        }
        Ok(command_output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cli_table::print_stdout;

    #[test]
    fn it_should_create_a_scratch_org() {
        let input = r#"{
  "status": 0,
  "result": {
    "username": "test@example.com",
    "scratchOrgInfo": {
      "attributes": {
        "type": "ScratchOrgInfo",
        "url": "/services/data/v62.0/sobjects/ScratchOrgInfo/2SRbm000000H9ZxGAK"
      },
      "Id": "1",
      "OwnerId": "1",
      "IsDeleted": false,
      "Name": "00000005",
      "CreatedDate": "2025-01-01T05:29:34.000+0000",
      "CreatedById": "1",
      "LastModifiedDate": "2025-01-01T05:29:49.000+0000",
      "LastModifiedById": "1",
      "SystemModstamp": "2025-01-01T05:29:49.000+0000",
      "LastViewedDate": "2025-01-01T05:29:49.000+0000",
      "LastReferencedDate": "2025-01-01T05:29:49.000+0000",
      "Edition": "Developer",
      "Username": null,
      "AdminEmail": null,
      "OrgName": "Demo company",
      "DurationDays": 7,
      "ConnectedAppConsumerKey": "PlatformCLI",
      "ConnectedAppCallbackUrl": "http://localhost:1717/OauthRedirect",
      "Namespace": null,
      "Features": "EnableSetPasswordInApi;API",
      "Country": null,
      "Language": null,
      "Package2AncestorIds": null,
      "SourceOrg": null,
      "HasSampleData": false,
      "Release": "Current",
      "SignupUsername": "test@test.com",
      "Status": "Active",
      "ErrorCode": null,
      "ScratchOrg": "1",
      "SignupInstance": "USA260S",
      "SignupCountry": "US",
      "SignupLanguage": "en_US",
      "SignupEmail": "test@test.com",
      "SignupTrialDays": 7,
      "LoginUrl": "https://test.my.salesforce.com",
      "Description": null,
      "ExpirationDate": "2025-01-08",
      "LastLoginDate": null,
      "DeletedBy": null,
      "DeletedDate": null
    },
    "authFields": {
      "instanceUrl": "https://test.my.salesforce.com",
      "orgId": "1",
      "username": "test@example.com",
      "loginUrl": "https://test.my.salesforce.com",
      "clientId": "PlatformCLI",
      "isDevHub": false,
      "created": "1735709374000",
      "expirationDate": "2025-01-08",
      "createdOrgInstance": "USA260S",
      "isScratch": true,
      "isSandbox": false,
      "tracksSource": true,
      "instanceApiVersion": "62.0",
      "instanceApiVersionLastRetrieved": "12/31/2024, 10:29:51 PM"
    },
    "warnings": [],
    "orgId": "00DO4000009XSLJMA4"
  },
  "warnings": [
    "Record types defined in the scratch org definition file will stop being capitalized by default in a future release.\nSet the `org-capitalize-record-types` config var to `true` to enforce capitalization."
  ]
}
"#;

        let mut cli = Cli::new();
        cli.mock_cli_output(String::from(input));
        let command_output = &cli.create_scratch_org(&"devhub".to_string(), &"alias".to_string());
        assert!(command_output.is_ok());
        assert!(matches!(
            command_output.as_ref().unwrap().result.as_ref().unwrap(),
            CliResult::CreateScratchOrgResult { .. }
        ));
        assert!(print_stdout(command_output.as_ref().unwrap().get_formatted_results()).is_ok());
    }
}
