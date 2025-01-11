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
#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PackageDirectory {
    dependencies: Option<Vec<Dependency>>,
    package: String,
    path: String,
    version_number: String,
    version_name: Option<String>,
    version_description: Option<String>,
    default: Option<bool>,
    unpackaged_metadata: Option<String>,
}
#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Dependency {
    package: String,
    version_number: Option<String>,
}

#[derive(Debug)]
pub struct SalesforceProjectConfig {
    name: String,
    packages: Vec<Package>,
}

#[derive(Debug)]
struct Version {
    name: Option<String>,
    number: String,
    description: Option<String>,
}

#[derive(Debug)]
pub struct Package {
    name: String,
    path: String,
    version: Version,
    unpackaged_metadata: Option<String>,
    dependencies: Option<Vec<PackageDependency>>,
}

impl Package {
    fn from(
        package_directory: &PackageDirectory,
        package_aliases: Option<HashMap<String, String>>,
    ) -> Package {
        let package_directory = package_directory.clone();
        let dependencies =
            Self::get_package_dependencies(package_directory.to_owned(), package_aliases);
        Package {
            name: package_directory.package,
            path: package_directory.path,
            version: Version {
                name: package_directory.version_name,
                number: package_directory.version_number,
                description: package_directory.version_description,
            },
            unpackaged_metadata: package_directory.unpackaged_metadata,
            dependencies,
        }
    }

    fn get_package_dependencies(
        package_directory: PackageDirectory,
        package_aliases: Option<HashMap<String, String>>,
    ) -> Option<Vec<PackageDependency>> {
        if package_directory.dependencies.is_some() {
            let mut package_dependencies = Vec::new();
            for dependency in package_directory.dependencies.unwrap().into_iter() {
                let mut package_dependency = PackageDependency::new();
                package_dependency.name = dependency.package.to_owned();

                // TODO: refactor
                if dependency.version_number.is_some() {
                    let dependency_name = format!(
                        "{}@{}",
                        dependency.package,
                        dependency.version_number.unwrap()
                    );
                    if let Some(version_id) =
                        package_aliases.to_owned().unwrap().get(&dependency_name)
                    {
                        package_dependency.id = version_id.to_string();
                    }
                    package_dependencies.push(package_dependency);
                } else {
                    if let Some(version_id) =
                        package_aliases.to_owned().unwrap().get(&dependency.package)
                    {
                        package_dependency.id = version_id.to_string();
                    }
                    package_dependencies.push(package_dependency);
                }
            }
            Some(package_dependencies)
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct PackageDependency {
    pub name: String,
    version: String,
    pub id: String,
}

impl PackageDependency {
    fn new() -> PackageDependency {
        PackageDependency {
            name: String::from(""),
            version: String::from(""),
            id: String::from(""),
        }
    }
}

impl SalesforceProjectConfig {
    pub fn get_name(&self) -> &String {
        &self.name
    }
    pub fn get_packages(&self) -> &Vec<Package> {
        &self.packages
    }
}

pub fn read(path: Option<String>) -> SalesforceProjectConfig {
    let project_json_path = path.unwrap_or(String::from("./sfdx-project.json"));
    let file = fs::read_to_string(project_json_path).expect("Did not find sfdx-project.json");
    let json: ProjectJson =
        serde_json::from_str(&file).expect("SFDX Project JSON is not in expected format");

    let mut project_config = SalesforceProjectConfig {
        name: json.name,
        packages: Vec::new(),
    };

    for package_directory in json.package_directories.into_iter() {
        let package = Package::from(
            &package_directory.to_owned(),
            json.package_aliases.to_owned(),
        );
        project_config.packages.push(package);
    }

    project_config
}

#[cfg(test)]
mod tests {
    use super::read;

    #[test]
    fn it_should_read_project_json() {
        let project_config = read(Some(String::from("tests/resources/sfdx-project.json")));
        assert_eq!(2, project_config.get_packages().len());

        assert_eq!("A", project_config.get_packages()[0].name);
        assert_eq!("1.0", project_config.get_packages()[0].version.number);
        // assert_eq!("04tB00000000000000", todo!());

        assert_eq!("B@2.0", project_config.get_packages()[1].name);
        // assert_eq!("", todo!());
        // assert_eq!("04tB00000000000001", todo!());
    }

    #[test]
    #[should_panic(expected = "Did not find sfdx-project.json")]
    fn it_should_not_find_sfdx_project_json() {
        read(Some(String::from("tests/resources/sfdx-project.json2")));
    }
}
