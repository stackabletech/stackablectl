use crate::{helm, helpers, VALID_OPERATORS_WITH_EXAMPLES};
use log::{debug, info};
use std::str::FromStr;

#[derive(Debug)]
pub struct Operator {
    pub name: String,
    pub version: Option<String>,
    pub example: Option<String>,
}

impl Operator {
    fn new(name: String, version: Option<String>) -> Result<Self, String> {
        match VALID_OPERATORS_WITH_EXAMPLES.get(&name) {
            None => Err(format!(
                "The operator {name} does not exist or stackablectl is to old to know the operator"
            )),
            Some(example) => Ok(Operator {
                name,
                version,
                example: example.map(|v| v.to_string()),
            }),
        }
    }

    pub fn deploy(&self, install_example: bool) {
        info!(
            "Deploying {} operator{}{}",
            self.name,
            match &self.version {
                Some(version) => format!(" in version {version}"),
                None => "".to_string(),
            },
            match &self.example {
                Some(_) if install_example => " with example",
                _ => "",
            }
        );

        let helm_release_name = format!("{}-operator", self.name);
        match &self.version {
            None => helm::install_helm_release_from_repo(
                &helm_release_name,
                "stackable-dev",
                &helm_release_name,
                None,
                true,
            ),
            Some(version) if version.ends_with("-nightly") => helm::install_helm_release_from_repo(
                &helm_release_name,
                "stackable-dev",
                &helm_release_name,
                Some(version),
                false,
            ),
            Some(version) if version.contains("-pr") => helm::install_helm_release_from_repo(
                &helm_release_name,
                "stackable-test",
                &helm_release_name,
                Some(version),
                false,
            ),
            Some(version) => helm::install_helm_release_from_repo(
                &helm_release_name,
                "stackable-stable",
                &helm_release_name,
                Some(version),
                false,
            ),
        }

        if install_example {
            if let Some(example) = &self.example {
                debug!(
                    "Installing the following example for {}: {example}",
                    self.name
                );
                helpers::execute_command(vec!["kubectl", "apply", "-f", example]);
            }
        }
    }
}

impl FromStr for Operator {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split('=').collect::<Vec<_>>();
        match parts[..] {
            [operator] => Operator::new(operator.to_string(), None),
            [operator, version] => Operator::new(operator.to_string(), Some(version.to_string())),
            _ => Err(format!("Could not parse the operator definition {s}")),
        }
    }
}
