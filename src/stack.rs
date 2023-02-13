use crate::{arguments::OutputType, helm, helm::HELM_REPOS, helpers, kind, kube, release, CliArgs};
use cached::proc_macro::cached;
use clap::{Parser, ValueHint};
use comfy_table::{
    presets::{NOTHING, UTF8_FULL},
    Cell, ContentArrangement, Table,
};
use indexmap::IndexMap;
use lazy_static::lazy_static;
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, ops::Deref, sync::Mutex};

lazy_static! {
    pub static ref STACK_FILES: Mutex<Vec<String>> = Mutex::new(vec![
//        "https://raw.githubusercontent.com/stackabletech/stackablectl/main/stacks/stacks-v2.yaml"
        "stacks/stacks-v2.yaml"
            .to_string(),
    ]);
}

#[derive(Parser)]
pub enum CliCommandStack {
    /// List all the available stacks
    #[command(alias("ls"))]
    List {
        #[arg(short, long, value_enum, default_value = "text")]
        output: OutputType,
    },
    /// Show details of a specific stack
    #[command(alias("desc"))]
    Describe {
        /// Name of the stack to describe
        #[arg(required = true, value_hint = ValueHint::Other)]
        stack: String,

        #[arg(short, long, value_enum, default_value = "text")]
        output: OutputType,
    },
    /// Install a specific stack
    #[command(alias("in"))]
    Install {
        /// Name of the stack to install
        #[arg(required = true, value_hint = ValueHint::Other)]
        stack: String,

        /// List of parameters to use when installing the stack.
        /// Every parameter need to have the format `<parameter>=<value>`, e.g. `adminPassword=secret123`.
        /// Multiple parameters can be specified.
        /// Use `stackable stack describe <stack>` to list the available parameters.
        #[arg(short, long)]
        parameters: Vec<String>,

        /// If specified, a local Kubernetes cluster consisting of 4 nodes (1 for control-plane and 3 workers) for testing purposes will be created.
        /// Kind is a tool to spin up a local Kubernetes cluster running on Docker on your machine.
        /// You need to have `docker` and `kind` installed.
        /// Have a look at our documentation on how to install `kind` at <https://docs.stackable.tech/home/getting_started.html#_installing_kubernetes_using_kind>
        #[arg(short, long)]
        kind_cluster: bool,

        /// Name of the kind cluster created if `--kind-cluster` is specified
        #[arg(
            long,
            default_value = "stackable-data-platform",
            requires = "kind_cluster",
            value_hint = ValueHint::Other,
        )]
        kind_cluster_name: String,
    },
}

impl CliCommandStack {
    pub async fn handle(&self) -> Result<(), Box<dyn Error>> {
        match self {
            CliCommandStack::List { output } => list_stacks(output).await?,
            CliCommandStack::Describe { stack, output } => describe_stack(stack, output).await?,
            CliCommandStack::Install {
                stack,
                parameters,
                kind_cluster,
                kind_cluster_name,
            } => {
                kind::handle_cli_arguments(*kind_cluster, kind_cluster_name)?;
                install_stack(stack, parameters).await?;
            }
        }
        Ok(())
    }
}

pub fn handle_common_cli_args(args: &CliArgs) {
    let mut stack_files = STACK_FILES.lock().unwrap();
    stack_files.extend_from_slice(&args.additional_stacks_file);
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Stacks {
    #[serde(with = "serde_yaml::with::singleton_map_recursive")]
    stacks: IndexMap<String, Stack>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Stack {
    description: String,
    stackable_release: String,
    stackable_operators: Vec<String>,
    #[serde(default)]
    labels: Vec<String>,
    #[serde(default)]
    manifests: Vec<HelmChartOrYaml>,
    #[serde(default)]
    parameters: Vec<StackParameter>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum HelmChartOrYaml {
    HelmChart(String),
    PlainYaml(String),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HelmChart {
    release_name: String,
    name: String,
    repo: HelmChartRepo,
    version: String,
    options: serde_yaml::Value,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HelmChartRepo {
    name: String,
    url: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StackParameter {
    pub name: String,
    pub description: String,
    pub default: String,
}

impl StackParameter {
    pub fn from_cli_parameters(
        stack_parameters: &[StackParameter],
        cli_parameters: &[String],
    ) -> Result<HashMap<String, String>, Box<dyn Error>> {
        let mut parameters: HashMap<String, String> = stack_parameters
            .iter()
            .map(|p| (p.name.clone(), p.default.clone()))
            .collect();

        for parameter in cli_parameters {
            let mut split = parameter.split('=');
            let name = split.next().ok_or("Can not parse parameter. Parameters need to have the format `<parameter>=<value>`, e.g. `adminPassword=secret123`.")?;
            let value = split.next().ok_or("Can not parse parameter. Parameters need to have the format `<parameter>=<value>`, e.g. `adminPassword=secret123`.")?;

            if !parameters.contains_key(name) {
                return Err(format!(
                    "Parameter {name} not known. Known parameters are {}",
                    stack_parameters
                        .iter()
                        .map(|p| p.name.clone())
                        .collect::<Vec<String>>()
                        .join(", ")
                )
                .into());
            }
            parameters.insert(name.to_string(), value.to_string());
        }

        Ok(parameters)
    }
}

async fn list_stacks(output_type: &OutputType) -> Result<(), Box<dyn Error>> {
    let output = get_stacks().await;
    match output_type {
        OutputType::Text => {
            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec![
                    Cell::new("Stack"),
                    Cell::new("Stackable release"),
                    Cell::new("Description"),
                ]);
            for (stack_name, stack) in output.stacks {
                table.add_row(vec![
                    Cell::new(stack_name),
                    Cell::new(stack.stackable_release),
                    Cell::new(stack.description),
                ]);
            }
            println!("{table}");
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

async fn describe_stack(stack_name: &str, output_type: &OutputType) -> Result<(), Box<dyn Error>> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Output {
        stack: String,
        description: String,
        stackable_release: String,
        stackable_operators: Vec<String>,
        labels: Vec<String>,
        parameters: Vec<StackParameter>,
    }

    let stack = get_stack(stack_name).await?;
    let output = Output {
        stack: stack_name.to_string(),
        description: stack.description,
        stackable_release: stack.stackable_release,
        stackable_operators: stack.stackable_operators,
        labels: stack.labels,
        parameters: stack.parameters,
    };

    match output_type {
        OutputType::Text => {
            let mut table = Table::new();
            table
                .load_preset(NOTHING)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .add_row(vec![Cell::new("Stack"), Cell::new(output.stack)])
                .add_row(vec![
                    Cell::new("Description"),
                    Cell::new(output.description),
                ])
                .add_row(vec![
                    Cell::new("Stackable release"),
                    Cell::new(output.stackable_release),
                ])
                .add_row(vec![
                    Cell::new("Stackable operators"),
                    Cell::new(output.stackable_operators.join(", ")),
                ])
                .add_row(vec![
                    Cell::new("Labels"),
                    Cell::new(output.labels.join(", ")),
                ]);
            println!("{table}");

            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec![
                    Cell::new("Parameter"),
                    Cell::new("Description"),
                    Cell::new("Default"),
                ]);
            for parameter in output.parameters {
                table.add_row(vec![
                    Cell::new(parameter.name),
                    Cell::new(parameter.description),
                    Cell::new(parameter.default),
                ]);
            }
            println!("{table}");
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

pub async fn install_stack(stack_name: &str, parameters: &[String]) -> Result<(), Box<dyn Error>> {
    info!("Installing stack {stack_name}");
    let stack = get_stack(stack_name).await?;
    let parameters = StackParameter::from_cli_parameters(&stack.parameters, parameters)?;

    release::install_release(&stack.stackable_release, &stack.stackable_operators, &[]).await?;

    info!("Installing components of stack {stack_name}");
    install_manifests(&stack.manifests, &parameters).await?;

    info!("Installed stack {stack_name}");
    Ok(())
}

pub async fn install_manifests(
    manifests: &[HelmChartOrYaml],
    parameters: &HashMap<String, String>,
) -> Result<(), Box<dyn Error>> {
    for manifest in manifests {
        match manifest {
            HelmChartOrYaml::HelmChart(helm_chart_file) => {
                let helm_chart =
                    helpers::read_from_url_or_file_with_templating(helm_chart_file, parameters)
                        .await
                        .map_err(|err| {
                            format!(
                                "Could not read helm chart from file \"{helm_chart_file}\": {err}"
                            )
                        })?;
                let helm_chart: HelmChart = serde_yaml::from_str(&helm_chart)?;
                debug!(
                    "Installing helm chart {} as {}",
                    helm_chart.name, helm_chart.release_name
                );
                HELM_REPOS
                    .lock()?
                    .insert(helm_chart.repo.name.clone(), helm_chart.repo.url.clone());

                let values_yaml = serde_yaml::to_string(&helm_chart.options)?;
                helm::install_helm_release_from_repo(
                    &helm_chart.release_name,
                    &helm_chart.release_name,
                    &helm_chart.repo.name,
                    &helm_chart.name,
                    Some(&helm_chart.version),
                    Some(&values_yaml),
                )?
            }
            HelmChartOrYaml::PlainYaml(yaml_url_or_file) => {
                debug!("Installing yaml manifest from {yaml_url_or_file}");
                let manifests =
                    helpers::read_from_url_or_file_with_templating(yaml_url_or_file, parameters)
                        .await
                        .map_err(|err| {
                            format!(
                            "Could not read stack manifests from file \"{yaml_url_or_file}\": {err}"
                        )
                        })?;
                kube::deploy_manifests(&manifests).await?;
            }
        }
    }

    Ok(())
}

/// Cached because of potential slow network calls
#[cached]
async fn get_stacks() -> Stacks {
    let mut all_stacks = IndexMap::new();
    let stack_files = STACK_FILES.lock().unwrap().deref().clone();
    for stack_file in stack_files {
        let yaml = helpers::read_from_url_or_file(&stack_file).await;
        match yaml {
            Ok(yaml) => match serde_yaml::from_str::<Stacks>(&yaml) {
                Ok(stacks) => all_stacks.extend(stacks.stacks),
                Err(err) => warn!("Failed to parse stack list from {stack_file}: {err}"),
            },
            Err(err) => {
                warn!("Could not read from stacks file \"{stack_file}\": {err}");
            }
        }
    }

    Stacks { stacks: all_stacks }
}

async fn get_stack(stack_name: &str) -> Result<Stack, Box<dyn Error>> {
    get_stacks()
    .await
        .stacks
        .remove(stack_name) // We need to remove to take ownership
        .ok_or_else(|| format!("Stack {stack_name} not found. Use `stackablectl stack list` to list the available stacks.").into())
}
