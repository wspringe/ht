use serde::Deserialize;
use std::{
    borrow::{Borrow, BorrowMut},
    collections::{BTreeMap, HashMap},
    fs::{self},
    ops::Deref,
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
pub struct Package {
    name: String,
    path: String,
    version_name: Option<String>,
    version_description: Option<String>,
    version_number: String,
    pub unpackaged_metadata: Option<String>,
    pub dependencies: Option<Vec<PackageDependency>>,
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
            version_name: package_directory.version_name,
            version_description: package_directory.version_description,
            version_number: package_directory.version_number,
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

                let mut package_dependency = PackageDependency::new();
                if dependency.version_number.is_some() {
                    let version_number = &dependency.version_number.unwrap();
                    let trimmed_version_number = Self::get_version_number_from(version_number);
                    package_dependency.version = Version::from(trimmed_version_number);
                    let dependency_name = format!("{}@{}", dependency.package, version_number);
                    if let Some(version_id) =
                        package_aliases.to_owned().unwrap().get(&dependency_name)
                    {
                        package_dependency.id = version_id.to_string();
                    }
                } else {
                    let version_number =
                        &dependency.package.split("@").collect::<Vec<&str>>()[1].to_string();
                    let trimmed_version_number = Self::get_version_number_from(version_number);

                    package_dependency.version = Version::from(trimmed_version_number);
                    if let Some(version_id) =
                        package_aliases.to_owned().unwrap().get(&dependency.package)
                    {
                        package_dependency.id = version_id.to_string();
                    }
                }
                package_dependencies.push(package_dependency);
            }
            Some(package_dependencies)
        } else {
            None
        }
    }

    fn get_version_number_from(source: &String) -> &str {
        source.split('-').next().unwrap()
    }
}

#[derive(Debug, Clone)]
struct Version {
    major: i32,
    minor: i32,
    patch: i32,
}

impl Version {
    fn new() -> Version {
        Version {
            major: 0,
            minor: 0,
            patch: 0,
        }
    }
    fn from(as_string: &str) -> Version {
        let version = as_string.split('.').collect::<Vec<&str>>();
        Version {
            major: version[0].parse().unwrap(),
            minor: version[1].parse().unwrap(),
            patch: version[2].parse().unwrap(),
        }
    }

    pub fn is_higher_than(&self, to_compare: &Version) -> bool {
        return self.major > to_compare.major
            || self.minor > to_compare.minor
            || self.patch > to_compare.patch;
    }
}

#[derive(Debug, Clone)]
pub struct PackageDependency {
    pub name: String,
    version: Version,
    pub id: String,
}

impl PackageDependency {
    fn new() -> PackageDependency {
        PackageDependency {
            name: String::from(""),
            version: Version::new(),
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

    pub fn get_dependencies(&mut self) -> Option<Vec<PackageDependency>> {
        let mut dependency_by_name: BTreeMap<String, PackageDependency> = BTreeMap::new();

        for package in self.packages.iter() {
            if let Some(vec) = &package.dependencies {
                for dependency in vec.iter() {
                    let dep = dependency.to_owned().clone();
                    let dep2 = dependency.clone();
                    let name = dependency.name.clone();
                    dependency_by_name
                        .entry(name)
                        .and_modify(|val| {
                            if dep.version.is_higher_than(&val.version) {
                                *val = dep;
                            }
                        })
                        .or_insert(dep2);
                }
            }
        }

        if dependency_by_name.is_empty() {
            None
        } else {
            let mut to_return: Vec<PackageDependency> = Vec::new();
            dependency_by_name.into_iter().for_each(|(_key, value)| {
                to_return.push(value.clone());
            });
            Some(to_return)
        }
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
        // assert_eq!("1.0", project_config.get_packages()[0].version.number);
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
