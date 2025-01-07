// TODO: refactor into separate modules for each command (makes it easier to handle cli command outputs)

use anyhow::anyhow;
use anyhow::Result;
use cli_table::format::Justify;
use cli_table::{Cell, Style, Table, TableStruct};
use enum_as_inner::EnumAsInner;
use serde::Deserialize;
use std::process::Stdio;
use std::{
    fmt::{self, Display, Formatter},
    process::Command,
};

pub trait SfCliResult {
    fn get_formatted_results(&self) -> TableStruct;
}

#[derive(Deserialize, Debug)]
struct RunTestResult {
    #[serde(rename = "Outcome")]
    outcome: String,
    #[serde(rename = "Message")]
    message: Option<String>,
    #[serde(rename = "MethodName")]
    method_name: String,
    #[serde(rename = "FullName")]
    full_name: String,
    #[serde(rename = "StackTrace")]
    stack_trace: Option<String>,
    #[serde(rename = "RunTime")]
    run_time: u32,
}
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct RunTestSummary {
    test_execution_time: String,
    failing: u32,
    fail_rate: String,
    tests_ran: u32,
    org_wide_coverage: String,
    test_run_coverage: String,
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
pub enum CliResult {
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
    #[serde(rename_all = "camelCase")]
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
    RunApexTestsResult {
        summary: RunTestSummary,
        tests: Vec<RunTestResult>,
    },
    PackageInstallResult {
        #[serde(rename = "Status")]
        status: String,
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
            CliResult::ExecuteAnonymousApexResult {
                compile_problem, ..
            } => vec![
                vec![
                    "Compilation Successful".cell(),
                    compile_problem.is_empty().cell().justify(Justify::Right),
                ],
                vec![
                    "Problems".cell(),
                    compile_problem.cell().justify(Justify::Right),
                ],
            ]
            .table()
            .title(vec![
                "Execute Anonymous Apex Script Results".cell().bold(true),
                "".cell(),
            ])
            .bold(true),
            CliResult::RunApexTestsResult { summary, tests } => vec![
                vec![
                    "Is Successful".cell(),
                    (summary.failing == 0).cell().justify(Justify::Right),
                ],
                vec![
                    "Code Coverage".cell(),
                    summary
                        .org_wide_coverage
                        .clone()
                        .cell()
                        .justify(Justify::Right),
                ],
                vec![
                    "Failures".cell(),
                    tests
                        .iter()
                        .map(|x| {
                            if x.outcome == "Fail" {
                                format!(
                                    "{full_name}: {stack_trace} - {message}\n",
                                    full_name = x.full_name,
                                    stack_trace = x.stack_trace.clone().unwrap(),
                                    message = x.message.clone().unwrap()
                                )
                            } else {
                                String::new()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                        .cell()
                        .justify(Justify::Right),
                ],
            ]
            .table()
            .title(vec!["Run Apex Tests Result".cell().bold(true), "".cell()])
            .bold(true),
            _ => unreachable!(),
        }
    }
}

#[derive(Deserialize)]
pub struct SfCliCommandOutput {
    name: Option<String>,
    message: Option<String>,
    pub result: Option<CliResult>,
    status: u32,
}

#[derive(Debug)]
pub struct SfCliError;

impl Display for SfCliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "something went wrong using the SF CLI")
    }
}

pub fn verify_cli_is_installed() -> Result<()> {
    match Command::new("sf").output() {
        Ok(_) => Ok(()),
        Err(..) => Err(anyhow!("SF CLI not found")),
    }
}
pub struct Cli {
    output: String,
    target_org: String,
}
impl Cli {
    pub fn new(target_org: String) -> Self {
        Cli {
            output: String::new(),
            target_org: target_org.clone(),
        }
    }

    fn mock_cli_output(&mut self, output: String) -> &mut Self {
        self.output = output;
        self
    }

    pub fn create_scratch_org(&mut self, devhub: &String) -> Result<SfCliCommandOutput> {
        let output = if self.output.is_empty() {
            let target_org = self.target_org.clone();
            self.get_output(vec![
                "org",
                "create",
                "scratch",
                "-v",
                devhub,
                "--definition-file",
                "config/project-scratch-def.json",
                "--alias",
                target_org.as_str(),
                "--set-default",
                "--json",
            ])?
        } else {
            self.output.clone()
        };

        let command_output: SfCliCommandOutput = serde_json::from_str(output.as_str())
            .expect("could not deserialize sf cli command output");
        if command_output.status != 0 {
            return Err(anyhow!(SfCliError).context(format!(
                "could not create scratch org: {}",
                command_output.message.unwrap(),
            )));
        }
        Ok(command_output)
    }

    fn get_output(&mut self, command_args: Vec<&str>) -> Result<String> {
        let command_output = Command::new("sf")
            .args(command_args)
            .stdout(Stdio::piped())
            .spawn()?;
        let output = command_output.wait_with_output();

        match output {
            Ok(x) => Ok(String::from_utf8(x.stdout)?),
            Err(e) => Err(anyhow!(SfCliError).context(e.to_string())),
        }
    }

    pub fn delete_old_scratch(&mut self) -> Result<SfCliCommandOutput> {
        let output = if self.output.is_empty() {
            let target_org = self.target_org.clone();
            self.get_output(vec![
                "org",
                "delete",
                "scratch",
                "--target-org",
                target_org.as_str(),
                "--no-prompt",
                "--json",
            ])?
        } else {
            self.output.clone()
        };

        let command_output: SfCliCommandOutput = serde_json::from_str(output.as_str())?;
        if command_output.status != 0 {
            return Err(anyhow!(SfCliError).context(format!(
                "could not delete scratch org: {}",
                command_output.message.unwrap(),
            )));
        }
        Ok(command_output)
    }

    pub fn auth_devhub(&mut self, path_to_auth_file: &str) -> Result<SfCliCommandOutput> {
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
        if command_output.status != 0 {
            return Err(anyhow!(SfCliError).context(format!(
                "could not authorize devhub: {}",
                command_output.message.unwrap(),
            )));
        }
        Ok(command_output)
    }

    pub fn project_deploy(&mut self, path: &str) -> Result<SfCliCommandOutput> {
        let output = if self.output.is_empty() {
            let target_org = self.target_org.clone();
            self.get_output(vec![
                "project",
                "deploy",
                "start",
                "-d",
                path,
                "--json",
                "-o",
                target_org.as_str(),
            ])?
        } else {
            self.output.clone()
        };

        let command_output: SfCliCommandOutput = serde_json::from_str(output.as_str())
            .expect("could not deserialize sf cli command output");
        if command_output.status != 0 {
            return Err(anyhow!(SfCliError).context(format!(
                "could not deploy metadata: {}",
                command_output.message.unwrap(),
            )));
        }
        Ok(command_output)
    }

    pub fn exec_anonymous(&mut self, path: &str) -> Result<SfCliCommandOutput> {
        let output = if self.output.is_empty() {
            let target_org = self.target_org.clone();
            self.get_output(vec![
                "apex",
                "run",
                "--file",
                path,
                "--json",
                "-o",
                target_org.as_str(),
            ])?
        } else {
            self.output.clone()
        };

        let command_output: SfCliCommandOutput = serde_json::from_str(output.as_str())
            .expect("could not deserialize sf cli command output");
        if command_output.status != 0 {
            return Err(anyhow!(SfCliError).context(format!(
                "could not execute anonymous apex: {}",
                command_output.message.unwrap(),
            )));
        }
        Ok(command_output)
    }

    pub fn run_tests(&mut self) -> Result<SfCliCommandOutput> {
        let output = if self.output.is_empty() {
            let target_org = self.target_org.clone();
            self.get_output(vec![
                "apex",
                "run",
                "test",
                "-c",
                "-l",
                "RunLocalTests",
                "-w",
                "60",
                "--json",
                "--target-org",
                target_org.as_str(),
            ])?
        } else {
            self.output.clone()
        };

        let command_output: SfCliCommandOutput = serde_json::from_str(output.as_str())
            .expect("could not deserialize sf cli command output");

        // I do not know why the status for this is 100
        if command_output.status != 100 {
            return Err(anyhow!(SfCliError).context(format!(
                "could not run apex tests: {}",
                command_output.message.unwrap(),
            )));
        }
        Ok(command_output)
    }

    // TODO: handle packages with keys
    pub fn install_package(&mut self, package_id: &str) -> Result<SfCliCommandOutput> {
        let output = if self.output.is_empty() {
            let target_org = self.target_org.clone();
            self.get_output(vec![
                "package",
                "install",
                "--package",
                package_id,
                "-w",
                "60",
                "--json",
                "-o",
                target_org.as_str(),
            ])?
        } else {
            self.output.clone()
        };

        let command_output: SfCliCommandOutput = serde_json::from_str(output.as_str())
            .expect("could not deserialize sf cli command output");
        if command_output.status != 0 {
            return Err(anyhow!(SfCliError).context(format!(
                "could not install package: {}",
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

        let mut cli = Cli::new(String::from("test"));
        cli.mock_cli_output(String::from(input));
        let command_output = &cli.create_scratch_org(&"devhub".to_string());
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

        let mut cli = Cli::new(String::from("test"));
        cli.mock_cli_output(String::from(input));
        let command_output = &cli.delete_old_scratch();
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

        let mut cli = Cli::new(String::from("test"));
        cli.mock_cli_output(String::from(input));
        let command_output = &cli.auth_devhub(&"path".to_string());
        assert!(command_output.is_ok());

        let result = command_output.as_ref().unwrap().result.as_ref();
        assert!(result.is_some());
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

        let mut cli = Cli::new(String::from("test"));
        cli.mock_cli_output(String::from(input));
        let command_output = &cli.project_deploy(&"path".to_string());
        assert!(command_output.is_ok());

        let result = command_output.as_ref().unwrap().result.as_ref();
        assert!(result.is_some());
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

    #[test]
    fn it_should_execute_anonymous_apex() {
        let input = r#"{
  "status": 0,
  "result": {
    "success": true,
    "compiled": true,
    "compileProblem": "",
    "exceptionMessage": "",
    "exceptionStackTrace": "",
    "line": -1,
    "column": -1,
    "logs": "logs"
  },
  "warnings": []
}
"#;

        let mut cli = Cli::new(String::from("test"));
        cli.mock_cli_output(String::from(input));
        let command_output = &cli.exec_anonymous(&"path".to_string());
        assert!(command_output.is_ok());

        let result = command_output.as_ref().unwrap().result.as_ref();
        assert!(result.is_some());
        assert!(matches!(
            result.unwrap(),
            CliResult::ExecuteAnonymousApexResult { .. }
        ));
        assert!(
            result
                .unwrap()
                .as_execute_anonymous_apex_result()
                .unwrap()
                .0
        );
        assert_eq!(
            "",
            result
                .unwrap()
                .as_execute_anonymous_apex_result()
                .unwrap()
                .1
        );
        assert!(print_stdout(command_output.as_ref().unwrap().get_formatted_results()).is_ok());
    }

    #[test]
    fn it_should_run_apex_tests() {
        let input = r#"{
  "status": 100,
  "result": {
    "summary": {
      "failRate": "50%",
      "failing": 1,
      "hostname": "https://ability-business-62982-dev-ed.scratch.my.salesforce.com",
      "orgId": "00DRt000008pQ2HMAU",
      "outcome": "Failed",
      "passRate": "50%",
      "passing": 1,
      "skipped": 0,
      "testRunId": "707Rt00000ZjIGa",
      "testStartTime": "2025-01-04T22:33:54.000Z",
      "testsRan": 2,
      "userId": "005Rt00000CcLDdIAN",
      "username": "test-vpfqm7c3a6cq@example.com",
      "commandTime": "169 ms",
      "testExecutionTime": "10 ms",
      "testTotalTime": "10 ms",
      "orgWideCoverage": "0%",
      "testRunCoverage": "0%"
    },
    "tests": [
      {
        "Id": "07MRt00000AbILBMA3",
        "QueueItemId": "709Rt00000ASqp7IAD",
        "StackTrace": null,
        "Message": null,
        "AsyncApexJobId": "707Rt00000ZjIGaIAN",
        "MethodName": "runTest",
        "Outcome": "Pass",
        "ApexClass": {
          "Id": "01pRt000009xaTnIAI",
          "Name": "TestClass",
          "NamespacePrefix": null
        },
        "RunTime": 8,
        "FullName": "TestClass.runTest"
      },
      {
        "Id": "07MRt00000AbILCMA3",
        "QueueItemId": "709Rt00000ASqp7IAD",
        "StackTrace": "Class.TestClass.runTest2: line 10, column 1",
        "Message": "System.AssertException: Assertion Failed: Expected: 2, Actual: 3",
        "AsyncApexJobId": "707Rt00000ZjIGaIAN",
        "MethodName": "runTest2",
        "Outcome": "Fail",
        "ApexClass": {
          "Id": "01pRt000009xaTnIAI",
          "Name": "TestClass",
          "NamespacePrefix": null
        },
        "RunTime": 2,
        "FullName": "TestClass.runTest2"
      }
    ],
    "coverage": {
      "coverage": [
        {
          "id": "01pRt000009wnrBIAQ",
          "name": "Test",
          "totalLines": 1,
          "lines": {
            "2": 0
          },
          "totalCovered": 0,
          "coveredPercent": 0
        }
      ],
      "records": [],
      "summary": {
        "totalLines": 1,
        "coveredLines": 0,
        "orgWideCoverage": "0%",
        "testRunCoverage": "0%"
      }
    }
  },
  "warnings": []
}
"#;

        let mut cli = Cli::new(String::from("test"));
        cli.mock_cli_output(String::from(input));
        let command_output = &cli.run_tests();
        assert!(command_output.is_ok());

        let result = command_output.as_ref().unwrap().result.as_ref();
        assert!(result.is_some());
        assert!(matches!(
            result.unwrap(),
            CliResult::RunApexTestsResult { .. }
        ));
        assert_eq!(
            1,
            result
                .unwrap()
                .as_run_apex_tests_result()
                .unwrap()
                .0
                .failing
        );
        assert_eq!(
            2,
            result.unwrap().as_run_apex_tests_result().unwrap().1.len()
        );
        assert!(print_stdout(command_output.as_ref().unwrap().get_formatted_results()).is_ok());
    }

    #[test]
    fn it_should_install_a_package() {
        let input = r#"{
  "status": 0,
  "result": {
    "attributes": {
      "type": "PackageInstallRequest",
      "url": "/services/data/v62.0/tooling/sobjects/PackageInstallRequest/0Hfbm0000028WXpCAM"
    },
    "Id": "0Hfbm0000028WXpCAM",
    "IsDeleted": false,
    "CreatedDate": "2025-01-05T22:35:49.000+0000",
    "SkipHandlers": null,
    "Status": "SUCCESS",
    "Errors": null
  },
  "warnings": []
}
"#;

        let mut cli = Cli::new(String::from("test"));
        cli.mock_cli_output(String::from(input));
        let command_output = &cli.install_package(&String::from("id"));
        assert!(command_output.is_ok());

        let result = command_output.as_ref().unwrap().result.as_ref();
        assert!(result.is_some());
        assert!(matches!(
            result.unwrap(),
            CliResult::PackageInstallResult { .. }
        ));
        assert_eq!(
            "SUCCESS",
            result.unwrap().as_package_install_result().unwrap()
        );
    }
}
