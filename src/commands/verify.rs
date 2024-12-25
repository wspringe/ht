use crate::utils::project_config::ProjectConfig;
use crate::utils::sf;

pub fn run(devhub: &Option<String>, delete_old: &bool, project_config: &ProjectConfig) {
    let devhub_alias = match devhub {
        Some(x) => x,
        None => &String::from("DevHub"),
    };

    if *delete_old {
        sf::delete_old_scratch(project_config.get_name());
    }
    sf::create_scratch_org(devhub_alias, project_config.get_name());

    if project_config.get_unpackaged_metadata_path().is_some() {
        sf::project_deploy(
            project_config
                .get_unpackaged_metadata_path()
                .as_ref()
                .unwrap(),
        );
    }

    // run scripts
    // run tests
    // display results
}
