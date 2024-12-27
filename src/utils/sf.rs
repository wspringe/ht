use std::process::Command;
// TODO: make this into a builder? return results from each fn

pub fn verify_cli_is_installed() {
    match Command::new("sf").spawn() {
        Ok(_) => (),
        Err(..) => {
            println!("SF CLI not found");
        }
    }
}

pub fn create_scratch_org(devhub: &String, alias: &String) {
    Command::new("sf")
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
        .spawn()
        .expect("Could not create scratch org");
}

pub fn delete_old_scratch(scratch_name: &String) {
    Command::new("sf")
        .args(["org", "delete", "scratch", "--target-org", scratch_name])
        .spawn()
        .expect("Could not delete old scratch org");
}

pub fn auth_devhub(path_to_auth_file: &String) {
    Command::new("sf")
        .args([
            "org",
            "login",
            "sfdx-url",
            "--sfdx-url-file",
            path_to_auth_file,
        ])
        .spawn()
        .expect("Could not authorize to a Dev Hub");
}

pub fn project_deploy(path: &String) {
    Command::new("sf")
        .args(["project", "deploy", "start", "-d", path])
        .spawn()
        .expect("Could not deploy metadata");
}

pub fn exec_anonymous(path: &String) {
    Command::new("sf")
        .args(["apex", "run", "--file", path])
        .spawn()
        .expect("Could not execute apex file");
}
