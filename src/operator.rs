use crate::{arguments::OutputType, helm, helm::HELM_REPOS, kind, AVAILABLE_OPERATORS};
use clap::{Parser, ValueHint};
use indexmap::IndexMap;
use log::{info, warn};
use serde::Serialize;
use std::{error::Error, str::FromStr};

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
        /// Name of the operator to describe
        #[clap(required = true, value_hint = ValueHint::Other)]
        operator: String,

        #[clap(short, long, arg_enum, default_value = "text")]
        output: OutputType,
    },
    /// Install one or multiple operators
    #[clap(alias("in"))]
    Install {
        /// Space separated list of operators to install.
        /// Must have the form `name[=version]` e.g. `superset`, `superset=0.3.0`, `superset=0.3.0-nightly` or `superset=0.3.0-pr123`.
        /// If no version is specified the latest nightly version - build from the main branch - will be used.
        /// You can get the available versions with `stackablectl operator list` or `stackablectl operator describe superset`
        #[clap(multiple_occurrences(true), required = true, value_hint = ValueHint::Other)]
        operators: Vec<Operator>,

        /// If specified, a local Kubernetes cluster consisting of 4 nodes for testing purposes will be created.
        /// Kind is a tool to spin up a local Kubernetes cluster running on Docker on your machine.
        /// You need to have `docker` and `kind` installed.
        /// Have a look at our documentation on how to install `kind` at <https://docs.stackable.tech/home/getting_started.html#_installing_kubernetes_using_kind>
        #[clap(short, long)]
        kind_cluster: bool,

        /// Name of the kind cluster created if `--kind-cluster` is specified
        #[clap(
            long,
            default_value = "stackable-data-platform",
            requires = "kind-cluster",
            value_hint = ValueHint::Other,
        )]
        kind_cluster_name: String,
    },
    /// Uninstall a operator
    #[clap(alias("un"))]
    Uninstall {
        /// Space separated list of operators to uninstall.
        #[clap(multiple_occurrences(true), required = true, value_hint = ValueHint::Other)]
        operators: Vec<String>,
    },
    /// List installed operators
    Installed {
        #[clap(short, long, arg_enum, default_value = "text")]
        output: OutputType,
    },
}

impl CliCommandOperator {
    pub async fn handle(&self) -> Result<(), Box<dyn Error>> {
        match self {
            CliCommandOperator::List { output } => list_operators(output).await?,
            CliCommandOperator::Describe { operator, output } => {
                describe_operator(operator, output).await?
            }
            CliCommandOperator::Install {
                operators,
                kind_cluster,
                kind_cluster_name,
            } => {
                kind::handle_cli_arguments(*kind_cluster, kind_cluster_name)?;
                for operator in operators {
                    operator.install()?;
                }
            }
            CliCommandOperator::Uninstall { operators } => uninstall_operators(operators),
            CliCommandOperator::Installed { output } => list_installed_operators(output)?,
        }

        Ok(())
    }
}

async fn list_operators(output_type: &OutputType) -> Result<(), Box<dyn Error>> {
    type Output = IndexMap<String, OutputOperatorEntry>;

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct OutputOperatorEntry {
        stable_versions: Vec<String>,
        test_versions: Vec<String>,
        dev_versions: Vec<String>,
    }

    let mut output: Output = IndexMap::new();
    for operator in AVAILABLE_OPERATORS {
        output.insert(
            operator.to_string(),
            OutputOperatorEntry {
                stable_versions: get_versions_from_repo(operator, "stackable-stable").await?,
                test_versions: get_versions_from_repo(operator, "stackable-test").await?,
                dev_versions: get_versions_from_repo(operator, "stackable-dev").await?,
            },
        );
    }

    match output_type {
        OutputType::Text => {
            println!("OPERATOR           STABLE VERSIONS");
            for (operator, operator_entry) in output {
                println!(
                    "{:18} {}",
                    operator,
                    operator_entry.stable_versions.join(", ")
                );
            }
        }
        OutputType::Json => {
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputType::Yaml => {
            println!("{}", serde_yaml::to_string(&output)?);
        }
    }

    Ok(())
}

async fn describe_operator(operator: &str, output_type: &OutputType) -> Result<(), Box<dyn Error>> {
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
        stable_versions: get_versions_from_repo(operator, "stackable-stable").await?,
        test_versions: get_versions_from_repo(operator, "stackable-test").await?,
        dev_versions: get_versions_from_repo(operator, "stackable-dev").await?,
    };

    match output_type {
        OutputType::Text => {
            println!("Operator:           {}", output.operator);
            println!("Stable versions:    {}", output.stable_versions.join(", "));
            println!("Test versions:      {}", output.test_versions.join(", "));
            println!("Dev versions:       {}", output.dev_versions.join(", "));
        }
        OutputType::Json => {
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputType::Yaml => {
            println!("{}", serde_yaml::to_string(&output)?);
        }
    }

    Ok(())
}

async fn get_versions_from_repo(
    operator: &str,
    helm_repo_name: &str,
) -> Result<Vec<String>, Box<dyn Error>> {
    let chart_name = format!("{operator}-operator");

    let helm_repo_url = HELM_REPOS
        .lock()?
        .get(helm_repo_name)
        .ok_or(format!(
            "Could not find a helm repo with the name {helm_repo_name}"
        ))?
        .to_string();

    let repo = helm::get_repo_index(helm_repo_url).await?;

    match repo.entries.get(&chart_name) {
        None => {
            warn!("Could not find {operator} operator (chart name {chart_name}) in helm repo {helm_repo_name}");
            Ok(vec![])
        }
        Some(versions) => Ok(versions
            .iter()
            .map(|entry| entry.version.clone())
            .rev()
            .collect()),
    }
}

pub fn uninstall_operators(operators: &Vec<String>) {
    for operator in operators {
        info!("Uninstalling {operator} operator");
        // TODO: Check if CRD objects of these products exist and warn if they do
        helm::uninstall_helm_release(format!("{operator}-operator").as_str())
    }
}

fn list_installed_operators(output_type: &OutputType) -> Result<(), Box<dyn Error>> {
    type Output = IndexMap<String, OutputInstalledOperatorEntry>;

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct OutputInstalledOperatorEntry {
        version: String,
        namespace: String,
        status: String,
        last_updated: String,
    }

    let output: Output = helm::helm_list_releases()?
        .into_iter()
        .filter(|release| {
            AVAILABLE_OPERATORS
                .iter()
                .any(|available| release.name == format!("{available}-operator"))
        })
        .map(|release| {
            (
                release.name.trim_end_matches("-operator").to_string(),
                OutputInstalledOperatorEntry {
                    version: release.version,
                    namespace: release.namespace,
                    status: release.status,
                    last_updated: release.last_updated,
                },
            )
        })
        .collect();

    match output_type {
        OutputType::Text => {
            println!("OPERATOR              VERSION         NAMESPACE                      STATUS           LAST UPDATED");
            for (operator, operator_entry) in output {
                println!(
                    "{:21} {:15} {:30} {:16} {}",
                    operator,
                    operator_entry.version,
                    operator_entry.namespace,
                    operator_entry.status,
                    operator_entry.last_updated
                );
            }
        }
        OutputType::Json => {
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputType::Yaml => {
            println!("{}", serde_yaml::to_string(&output)?);
        }
    }

    Ok(())
}

#[derive(Debug)]
pub struct Operator {
    pub name: String,
    pub version: Option<String>,
}

impl Operator {
    pub fn new(name: String, version: Option<String>) -> Result<Self, String> {
        if !AVAILABLE_OPERATORS.contains(&name.as_str()) {
            Err(format!(
                "The operator {name} does not exist or stackablectl is to old to know the operator"
            ))
        } else {
            Ok(Operator { name, version })
        }
    }

    pub fn install(&self) -> Result<(), Box<dyn Error>> {
        info!(
            "Installing {} operator{}",
            self.name,
            match &self.version {
                Some(version) => format!(" in version {version}"),
                None => "".to_string(),
            },
        );

        let helm_release_name = format!("{}-operator", self.name);
        let helm_repo_name = match &self.version {
            None => "stackable-dev",
            Some(version) if version.ends_with("-nightly") => "stackable-dev",
            Some(version) if version.contains("-pr") => "stackable-test",
            Some(_) => "stackable-stable",
        };

        helm::install_helm_release_from_repo(
            &self.name,
            &helm_release_name,
            helm_repo_name,
            &helm_release_name,
            self.version.as_deref(),
            None,
        )?;

        Ok(())
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
