use serde::Deserialize;
use std::{
    collections::HashMap,
    fs::{self},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProjectJson {
    name: String,
    package_directories: Vec<PackageDirectory>,
    package_aliases: Option<HashMap<String, String>>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PackageDirectory {
    #[serde(default)]
    dependencies: Option<Vec<Dependency>>,
    package: String,
    path: String,
    version_number: String,
    default: Option<bool>,
    unpackaged_metadata: Option<String>,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Dependency {
    package: String,
    version_number: Option<String>,
}

#[derive(Debug)]
pub struct ProjectConfig {
    name: String,
    paths: Vec<String>,
    packages: Vec<Package>,
    unpackaged_metadata_path: Option<String>,
}

#[derive(Debug)]
pub struct Package {
    name: String,
    version: String,
    id: String,
}

impl ProjectConfig {
    pub fn get_name(&self) -> &String {
        &self.name
    }
    pub fn get_packages(&self) -> &Vec<Package> {
        &self.packages
    }
    pub fn get_unpackaged_metadata_path(&self) -> &Option<String> {
        &self.unpackaged_metadata_path
    }
    pub fn get_paths(&self) -> &Vec<String> {
        &self.paths
    }
}

pub fn read(path: Option<String>) -> ProjectConfig {
    let project_json_path = path.unwrap_or(String::from("./sfdx-project.json"));
    let file = fs::read_to_string(project_json_path).expect("Did not find sfdx-project.json");
    let json: ProjectJson =
        serde_json::from_str(&file).expect("SFDX Project JSON is not in expected format");

    let mut project_config = ProjectConfig {
        name: json.name,
        paths: vec![],
        packages: Vec::new(),
        unpackaged_metadata_path: None,
    };

    for package_directory in json.package_directories.into_iter() {
        for dependency in package_directory
            .dependencies
            .unwrap_or_default()
            .into_iter()
        {
            let package_aliases = json.package_aliases.clone().unwrap();
            if package_aliases.contains_key(&dependency.package) {
                let package_id = package_aliases.get(&dependency.package).unwrap();
                project_config.packages.push(Package {
                    name: dependency.package,
                    version: dependency.version_number.unwrap_or_default(),
                    id: String::from(package_id),
                })
            } else {
                project_config.packages.push(Package {
                    name: dependency.package,
                    version: dependency.version_number.unwrap_or_default(),
                    id: String::from(""),
                });
            }
        }
    }

    return project_config;
}

#[cfg(test)]
mod tests {
    use super::read;

    #[test]
    fn it_should_read_project_json() {
        let project_config = read(Some(String::from("tests/resources/sfdx-project.json")));
        assert_eq!(2, project_config.get_packages().len());

        assert_eq!("A", project_config.get_packages()[0].name);
        assert_eq!("1.0", project_config.get_packages()[0].version);
        assert_eq!("04tB00000000000000", project_config.get_packages()[0].id);

        assert_eq!("B@2.0", project_config.get_packages()[1].name);
        assert_eq!("", project_config.get_packages()[1].version);
        assert_eq!("04tB00000000000001", project_config.get_packages()[1].id);
    }

    #[test]
    #[should_panic(expected = "Did not find sfdx-project.json")]
    fn it_should_not_find_sfdx_project_json() {
        read(Some(String::from("tests/resources/sfdx-project.json2")));
    }
}
