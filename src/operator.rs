use crate::arguments::OutputType;
use crate::helm::HELM_REPOS;
use crate::{helm, kind, AVAILABLE_OPERATORS};
use clap::Parser;
use indexmap::IndexMap;
use log::{info, warn};
use serde::Serialize;
use std::str::FromStr;

#[derive(Parser)]
pub enum CliCommandOperator {
    /// List all the available operators
    #[clap(alias("ls"))]
    List {
        #[clap(short, long, arg_enum, default_value = "text")]
        output: OutputType,
    },
    /// Show details of a specific operator
    #[clap(alias("desc"))]
    Describe {
        operator: String,
        #[clap(short, long, arg_enum, default_value = "text")]
        output: OutputType,
    },
    /// Install a operator
    #[clap(alias("in"))]
    Install {
        /// Space separated list of operators to install.
        /// Must have the form `name[=version]` e.g. `superset`, `superset=0.3.0`, `superset=0.3.0-nightly` or `superset=0.3.0-pr123`.
        /// You can get the available versions with `stackablectl operator list` or `stackablectl operator describe superset`
        #[clap(multiple_occurrences(true), required = true)]
        operators: Vec<Operator>,

        /// If this argument is specified a local kubernetes cluster for testing purposes is created.
        /// Kind is a tool to spin up a local kubernetes cluster running on docker on your machine.
        /// This scripts creates such a cluster consisting of 4 nodes to test the Stackable Data Platform.
        /// The default cluster name is `stackable-data-platform` which can be overwritten by specifiing the cluster name after `--kind-cluster`
        /// You need to have `docker` and `kind` installed. Have a look at the README at https://github.com/stackabletech/stackablectl on how to install them
        #[clap(short, long)]
        kind_cluster: Option<Option<String>>,
    },
    /// Uninstall a operator
    #[clap(alias("un"))]
    Uninstall {
        /// Space separated list of operators to uninstall.
        #[clap(multiple_occurrences(true), required = true)]
        operators: Vec<String>,
    },
}

impl CliCommandOperator {
    pub fn handle(&self) {
        match self {
            CliCommandOperator::List { output } => list_operators(output),
            CliCommandOperator::Describe { operator, output } => {
                describe_operator(operator, output)
            }
            CliCommandOperator::Install {
                operators,
                kind_cluster,
            } => {
                kind::handle_cli_arguments(kind_cluster);
                for operator in operators {
                    operator.install();
                }
            }
            CliCommandOperator::Uninstall { operators } => uninstall_operators(operators),
        }
    }
}

fn list_operators(output_type: &OutputType) {
    type Output = IndexMap<String, OutputOperatorEntry>;

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct OutputOperatorEntry {
        stable_versions: Vec<String>,
        test_versions: Vec<String>,
        dev_versions: Vec<String>,
    }

    let mut output = Output::new();
    for operator in AVAILABLE_OPERATORS {
        output.insert(
            operator.to_string(),
            OutputOperatorEntry {
                stable_versions: get_versions_from_repo(operator, "stackable-stable"),
                test_versions: get_versions_from_repo(operator, "stackable-test"),
                dev_versions: get_versions_from_repo(operator, "stackable-dev"),
            },
        );
    }

    match output_type {
        OutputType::Text => {
            println!("OPERATOR           STABLE VERSIONS");
            for (operator, operator_entry) in output.iter() {
                println!(
                    "{:18} {}",
                    operator,
                    operator_entry.stable_versions.join(", ")
                );
            }
        }
        OutputType::Json => {
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        OutputType::Yaml => {
            println!("{}", serde_yaml::to_string(&output).unwrap());
        }
    }
}

fn describe_operator(operator: &str, output_type: &OutputType) {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Output {
        operator: String,
        stable_versions: Vec<String>,
        test_versions: Vec<String>,
        dev_versions: Vec<String>,
    }
    let output = Output {
        operator: operator.to_string(),
        stable_versions: get_versions_from_repo(operator, "stackable-stable"),
        test_versions: get_versions_from_repo(operator, "stackable-test"),
        dev_versions: get_versions_from_repo(operator, "stackable-dev"),
    };

    match output_type {
        OutputType::Text => {
            println!("Operator:           {}", output.operator);
            println!("Stable versions:    {}", output.stable_versions.join(", "));
            println!("Test versions:      {}", output.test_versions.join(", "));
            println!("Dev versions:       {}", output.dev_versions.join(", "));
        }
        OutputType::Json => {
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        OutputType::Yaml => {
            println!("{}", serde_yaml::to_string(&output).unwrap());
        }
    }
}

fn get_versions_from_repo(operator: &str, helm_repo_name: &str) -> Vec<String> {
    let chart_name = format!("{operator}-operator");

    let repo =
        helm::get_repo_index(HELM_REPOS.lock().unwrap().get(helm_repo_name).unwrap_or_else(|| {
            panic!("Could not find a helm repo with the name {helm_repo_name}")
        }).to_string());
    match repo.entries.get(&chart_name) {
        None => {
            warn!("Could not find {operator} operator (chart name {chart_name}) in helm repo {helm_repo_name}");
            vec![]
        }
        Some(versions) => versions
            .iter()
            .map(|entry| entry.version.clone())
            .rev()
            .collect(),
    }
}

fn uninstall_operators(operators: &Vec<String>) {
    for operator in operators {
        info!("Uninstalling {operator} operator");
        helm::uninstall_helm_release(format!("{operator}-operator").as_str())
    }
}

#[derive(Debug)]
pub struct Operator {
    pub name: String,
    pub version: Option<String>,
}

impl Operator {
    fn new(name: String, version: Option<String>) -> Result<Self, String> {
        if !AVAILABLE_OPERATORS.contains(&name.as_str()) {
            Err(format!(
                "The operator {name} does not exist or stackablectl is to old to know the operator"
            ))
        } else {
            Ok(Operator { name, version })
        }
    }

    pub fn install(&self) {
        info!(
            "Installing {} operator{}",
            self.name,
            match &self.version {
                Some(version) => format!(" in version {version}"),
                None => "".to_string(),
            },
        );

        let helm_release_name = format!("{}-operator", self.name);
        match &self.version {
            None => helm::install_helm_release_from_repo(
                &self.name,
                &helm_release_name,
                "stackable-dev",
                &helm_release_name,
                None,
            ),
            Some(version) if version.ends_with("-nightly") => helm::install_helm_release_from_repo(
                &self.name,
                &helm_release_name,
                "stackable-dev",
                &helm_release_name,
                Some(version),
            ),
            Some(version) if version.contains("-pr") => helm::install_helm_release_from_repo(
                &self.name,
                &helm_release_name,
                "stackable-test",
                &helm_release_name,
                Some(version),
            ),
            Some(version) => helm::install_helm_release_from_repo(
                &self.name,
                &helm_release_name,
                "stackable-stable",
                &helm_release_name,
                Some(version),
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
