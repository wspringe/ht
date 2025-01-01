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
struct ScratchOrgInfo {
    Id: String,
    Features: String,
}

#[derive(Deserialize)]
#[serde(untagged)]
// #[serde(rename_all = "camelCase")]
enum CliResult {
    CreateScratchOrgResult {
        username: String,
        scratchOrgInfo: ScratchOrgInfo,
        orgId: String,
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
                scratchOrgInfo,
                ..
            } => vec![
                vec!["Id".cell(), scratchOrgInfo.Id.clone().cell().justify(Justify::Right)],
                vec!["Username".cell(), username.cell().justify(Justify::Right)],
            ]
            .table()
            .title(vec!["Scratch Org".cell().bold(true), "Test".cell()])
            .bold(true),
            _ => {vec![
                vec!["Id".cell(), "id".cell().justify(Justify::Right)],
            ]
                .table()
                .title(vec!["Scratch Org".cell().bold(true)])
                .bold(true)},
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
        if !self.output.is_empty() {
            let output = self.output.clone();
            let command_output: SfCliCommandOutput = serde_json::from_str(output.as_str())
                .expect("could not deserialize sf cli command output");
            if command_output.name.contains("Error") {
                return Err(anyhow!(SfCliError).context(format!(
                    "could not create scratch org: {}",
                    command_output.message.unwrap(),
                )));
            }
            Ok(command_output)
        } else {
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

#[cfg(test)]
mod tests {
    use super::*;
    use cli_table::print_stdout;

    #[test]
    fn it_should_create_a_scratch_org() {
        let mut cli = Cli::new();
        let input = r#"{
        "name": "x",
  "status": 0,
  "result": {
    "username": "test-blpnvdpjpiyz@example.com",
    "scratchOrgInfo": {
      "attributes": {
        "type": "ScratchOrgInfo",
        "url": "/services/data/v62.0/sobjects/ScratchOrgInfo/2SRbm000000H9ZxGAK"
      },
      "Id": "2SRbm000000H9ZxGAK",
      "OwnerId": "005bm000004AABtAAO",
      "IsDeleted": false,
      "Name": "00000005",
      "CreatedDate": "2025-01-01T05:29:34.000+0000",
      "CreatedById": "005bm000004AABtAAO",
      "LastModifiedDate": "2025-01-01T05:29:49.000+0000",
      "LastModifiedById": "005bm000004A6XQAA0",
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
      "SignupUsername": "test-blpnvdpjpiyz@example.com",
      "Status": "Active",
      "ErrorCode": null,
      "ScratchOrg": "00DO4000009XSLJ",
      "SignupInstance": "USA260S",
      "AuthCode": "aPrxtSLWzVcgKiTAkeMEf3qU2f1._QkXKCZDoaBduj7jT1yZb2iiwCoBwGmtdDrl6wKtDDcXlA==",
      "SignupCountry": "US",
      "SignupLanguage": "en_US",
      "SignupEmail": "wesdemonnic@gmail.com",
      "SignupTrialDays": 7,
      "LoginUrl": "https://energy-dream-6326-dev-ed.scratch.my.salesforce.com",
      "Description": null,
      "ExpirationDate": "2025-01-08",
      "LastLoginDate": null,
      "DeletedBy": null,
      "DeletedDate": null
    },
    "authFields": {
      "accessToken": "7c8d3115deb23e0d0630c7a14945d346acc9f5f975b375728a04282edcbc5f23865474cf5130b3d888565308b226db7e9d912250ced738ac630ae1dbc4c7f468ba9c7115af9778b01146f5c7f956e7c6e422fcf4f3e06430eaf171f5b48a3d5252a6441f7fdfa058383bd88738cfb8abfb7cdd383c40:881b9a4bfb9bbe7437c760a9bc8ddd22",
      "instanceUrl": "https://energy-dream-6326-dev-ed.scratch.my.salesforce.com",
      "orgId": "00DO4000009XSLJMA4",
      "username": "test-blpnvdpjpiyz@example.com",
      "loginUrl": "https://energy-dream-6326-dev-ed.scratch.my.salesforce.com",
      "refreshToken": "b113a724e595e1dbe5b172d98c6e2d2bc0960cad9f2437de103c302b574b1b9d008251b05e70e21ef4a0d2999c38fea73d7f65b1a8d3b4ee5ddd7bd8583dbd3535313b5de4dd3b366b8b3b50bfb8ba4a942d3c23f0d4dffbb5d1c027df:60fc87779d59d9649bd0d3aaa3228fc4",
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
        cli.mock_cli_output(String::from(input));
        let output = cli.create_scratch_org(&"devhub".to_string(), &"alias".to_string());
        let s = print_stdout(output.unwrap().result.unwrap().get_formatted_results());
        assert!(s.is_ok());
    }
}
