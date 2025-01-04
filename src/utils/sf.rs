// TODO: refactor into separate modules for each command (makes it easier to handle cli command outputs)

use anyhow::anyhow;
use anyhow::Result;
use cli_table::format::Justify;
use cli_table::{Cell, Style, Table, TableStruct};
use enum_as_inner::EnumAsInner;
use serde::Deserialize;
use std::{
    fmt::{self, Display, Formatter},
    process::Command,
};

trait SfCliResult {
    fn get_formatted_results(&self) -> TableStruct;
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TestResult {
    message: String,
    method_name: String,
    name: String,
    stack_trace: String,
    time: u32,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RunTestResult {
    failures: Vec<TestResult>,
    successes: Vec<TestResult>,
    num_failures: u32,
    num_tests_run: u32,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct MetadataComponent {
    component_type: String,
    full_name: String,
    problem: Option<String>,
    success: bool,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct DeployDetails {
    component_successes: Vec<MetadataComponent>,
    component_failures: Vec<MetadataComponent>,
    run_test_result: RunTestResult,
}

#[derive(Deserialize, Debug)]
struct ScratchOrgInfo {
    #[serde(rename = "Id")]
    id: String,
    #[serde(rename = "Features")]
    features: String,
}

#[derive(Deserialize, EnumAsInner, Debug)]
#[serde(untagged)]
enum CliResult {
    CreateScratchOrgResult {
        username: String,
        #[serde(rename = "scratchOrgInfo")]
        scratch_org_info: ScratchOrgInfo,
        #[serde(rename = "orgId")]
        org_id: String,
    },
    AuthorizeResult {
        username: String,
        #[serde(rename = "instanceUrl")]
        instance_url: String,
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
    DeleteScratchOrgResult {
        username: String,
        #[serde(rename = "orgId")]
        org_id: String,
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
                    (details.component_failures.len() == 0)
                        .cell()
                        .justify(Justify::Right),
                ],
                vec![
                    "Problems".cell(),
                    details
                        .component_failures
                        .iter()
                        .map(|x| x.problem.as_ref().unwrap().as_str())
                        .collect::<Vec<_>>()
                        .join("\n")
                        .cell()
                        .justify(Justify::Right),
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

        let result = command_output.as_ref().unwrap().result.as_ref();
        assert!(result.is_some());
        assert!(matches!(
            result.unwrap(),
            CliResult::CreateScratchOrgResult { .. }
        ));
        assert_eq!(
            "test@example.com",
            result.unwrap().as_create_scratch_org_result().unwrap().0
        );
        assert!(print_stdout(command_output.as_ref().unwrap().get_formatted_results()).is_ok());
    }

    #[test]
    fn it_should_delete_a_scratch_org() {
        let input = r#"{
          "status": 0,
          "result": {
            "username": "test@example.com",
            "orgId": "001"
          },
          "warnings": []
        }
"#;

        let mut cli = Cli::new();
        cli.mock_cli_output(String::from(input));
        let command_output = &cli.delete_old_scratch(&"test".to_string());
        assert!(command_output.is_ok());

        let result = command_output.as_ref().unwrap().result.as_ref();
        assert!(result.is_some());
        assert!(matches!(
            result.unwrap(),
            CliResult::DeleteScratchOrgResult { .. }
        ));
        assert_eq!(
            "test@example.com",
            result.unwrap().as_delete_scratch_org_result().unwrap().0
        );
        assert!(print_stdout(command_output.as_ref().unwrap().get_formatted_results()).is_ok());
    }

    #[test]
    fn it_should_authorize_an_org() {
        let input = r#"{
  "status": 0,
  "result": {
    "accessToken": "token",
    "instanceUrl": "https://test.salesforce.com",
    "orgId": "001",
    "username": "test.com.sandbox",
    "loginUrl": "https://login.salesforce.com",
    "refreshToken": "refreshToken",
    "clientId": "PlatformCLI",
    "isDevHub": true,
    "instanceApiVersion": "62.0",
    "instanceApiVersionLastRetrieved": "1/3/2025, 9:29:08 PM"
  },
  "warnings": []
}
"#;

        let mut cli = Cli::new();
        cli.mock_cli_output(String::from(input));
        let command_output = &cli.auth_devhub(&"path".to_string());
        assert!(command_output.is_ok());

        let result = command_output.as_ref().unwrap().result.as_ref();
        assert!(result.is_some());
        println!("what is it, {:?}", result.unwrap());
        assert!(matches!(result.unwrap(), CliResult::AuthorizeResult { .. }));
        assert_eq!(
            "test.com.sandbox",
            result.unwrap().as_authorize_result().unwrap().0
        );
        assert!(print_stdout(command_output.as_ref().unwrap().get_formatted_results()).is_ok());
    }

    #[test]
    fn it_should_deploy_metadata() {
        let input = r#"{
  "status": 0,
  "result": {
    "checkOnly": false,
    "completedDate": "2025-01-04T07:44:01.000Z",
    "createdBy": "005Rt00000CcLDd",
    "createdByName": "User User",
    "createdDate": "2025-01-04T07:44:00.000Z",
    "details": {
      "componentSuccesses": [
        {
          "changed": true,
          "componentType": "ApexClass",
          "created": true,
          "createdDate": "2025-01-04T07:44:01.000Z",
          "deleted": false,
          "fileName": "classes/Test.cls",
          "fullName": "Test",
          "id": "01pRt000009wnrBIAQ",
          "success": true
        },
        {
          "changed": true,
          "componentType": "",
          "created": false,
          "createdDate": "2025-01-04T07:44:01.000Z",
          "deleted": false,
          "fileName": "package.xml",
          "fullName": "package.xml",
          "success": true
        }
      ],
      "runTestResult": {
        "numFailures": 0,
        "numTestsRun": 0,
        "totalTime": 0,
        "codeCoverage": [],
        "codeCoverageWarnings": [],
        "failures": [],
        "flowCoverage": [],
        "flowCoverageWarnings": [],
        "successes": []
      },
      "componentFailures": []
    },
    "done": true,
    "id": "0AfRt00000PqprFKAR",
    "ignoreWarnings": false,
    "lastModifiedDate": "2025-01-04T07:44:01.000Z",
    "numberComponentErrors": 0,
    "numberComponentsDeployed": 1,
    "numberComponentsTotal": 1,
    "numberTestErrors": 0,
    "numberTestsCompleted": 0,
    "numberTestsTotal": 0,
    "rollbackOnError": true,
    "runTestsEnabled": false,
    "startDate": "2025-01-04T07:44:00.000Z",
    "status": "Succeeded",
    "success": true,
    "files": [
      {
        "fullName": "Test",
        "type": "ApexClass",
        "state": "Created",
        "filePath": "test/force-app/main/default/classes/Test.cls"
      },
      {
        "fullName": "Test",
        "type": "ApexClass",
        "state": "Created",
        "filePath": "test/force-app/main/default/classes/Test.cls-meta.xml"
      }
    ],
    "zipSize": 791,
    "zipFileCount": 3,
    "deployUrl": "https://ability-business-62982-dev-ed.scratch.my.salesforce.com/lightning/setup/DeployStatus/page?address=%2Fchangemgmt%2FmonitorDeploymentsDetails.apexp%3FasyncId%3D0AfRt00000PqprFKAR%26retURL%3D%252Fchangemgmt%252FmonitorDeployment.apexp"
  },
  "warnings": []
}
"#;

        let mut cli = Cli::new();
        cli.mock_cli_output(String::from(input));
        let command_output = &cli.project_deploy(&"path".to_string());
        assert!(command_output.is_ok());

        let result = command_output.as_ref().unwrap().result.as_ref();
        assert!(result.is_some());
        println!("what is it, {:?}", result.unwrap());
        assert!(matches!(
            result.unwrap(),
            CliResult::ProjectDeployResult { .. }
        ));
        assert_eq!(
            2,
            result
                .unwrap()
                .as_project_deploy_result()
                .unwrap()
                .component_successes
                .len()
        );
        assert!(print_stdout(command_output.as_ref().unwrap().get_formatted_results()).is_ok());
    }
}
