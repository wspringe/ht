use crate::utils::project_config::ProjectConfig;
use crate::utils::{project, sf};

pub fn run(devhub: &Option<String>, delete_old: &bool, project_config: &ProjectConfig) {
    let devhub_alias = match devhub {
        Some(x) => x,
        None => &String::from("DevHub"),
    };

    if *delete_old {
        sf::delete_old_scratch(project_config.get_name());
    }
    sf::create_scratch_org(devhub_alias, project_config.get_name());

    // deploy unpackaged metadata if unspecified (should be before or after?)
    if project_config.get_unpackaged_metadata_path().is_some() {
        sf::project_deploy(
            project_config
                .get_unpackaged_metadata_path()
                .as_ref()
                .unwrap(),
        );
    }

    project::exec_predeploy_scripts();

    // deploy metadata
    for path in project_config.get_paths() {
        sf::project_deploy(path);
    }

    project::exec_postdeploy_scripts();

    // run tests
    sf::run_tests();
    // display results
    // TODO: implement a display for everything things that happened this run (scipts run, test results, paths deployed;  option for more verbose/debug results like every metadata deployed)
}
