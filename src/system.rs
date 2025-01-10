use std::process::Command;

pub fn exec_script(path: &String) {
    Command::new("sh")
        .arg(path)
        .spawn()
        .expect("Could not execute shell script");
}
