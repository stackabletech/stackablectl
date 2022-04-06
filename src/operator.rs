use crate::helm::{HELM_DEV_REPO_URL, HELM_STABLE_REPO_URL, HELM_TEST_REPO_URL};
use crate::{helm, VALID_OPERATORS_WITH_EXAMPLES};
use log::info;
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

    pub fn install(&self) {
        info!(
            "Deploying operator {} in {} {}",
            self.name,
            match &self.version {
                None => "development version".to_string(),
                Some(version) => format!("version {version}"),
            },
            match &self.example {
                None => "without example".to_string(),
                Some(example) => format!("with example {example}"),
            }
        );

        let helm_release_name = format!("{}-operator", self.name);
        match &self.version {
            None => {
                helm::install_helm_release(&helm_release_name, HELM_DEV_REPO_URL, vec!["--devel"])
            }
            Some(version) if version.ends_with("-nightly") => helm::install_helm_release(
                &helm_release_name,
                HELM_DEV_REPO_URL,
                vec!["--version", version],
            ),
            Some(version) if version.contains("-pr") => helm::install_helm_release(
                &helm_release_name,
                HELM_TEST_REPO_URL,
                vec!["--version", version],
            ),
            Some(version) => helm::install_helm_release(
                &helm_release_name,
                HELM_STABLE_REPO_URL,
                vec!["--version", version],
            ),
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
