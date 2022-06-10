use crate::{arguments::OutputType, helm, helm::HELM_REPOS, helpers, kind, kube, release, CliArgs};
use cached::proc_macro::cached;
use clap::Parser;
use indexmap::IndexMap;
use lazy_static::lazy_static;
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::{ops::Deref, process::exit, sync::Mutex};

lazy_static! {
    pub static ref STACK_FILES: Mutex<Vec<String>> = Mutex::new(vec![
        "https://raw.githubusercontent.com/stackabletech/stackablectl/main/stacks.yaml".to_string(),
    ]);
}

#[derive(Parser)]
pub enum CliCommandStack {
    /// List all the available stack
    #[clap(alias("ls"))]
    List {
        #[clap(short, long, arg_enum, default_value = "text")]
        output: OutputType,
    },
    /// Show details of a specific stack
    #[clap(alias("desc"))]
    Describe {
        stack: String,

        #[clap(short, long, arg_enum, default_value = "text")]
        output: OutputType,
    },
    /// Install a specific stack
    #[clap(alias("in"))]
    Install {
        /// Name of the stack to install
        #[clap(required = true)]
        stack: String,

        /// If specified a local kubernetes cluster consisting of 4 nodes for testing purposes will be created.
        /// Kind is a tool to spin up a local kubernetes cluster running on docker on your machine.
        /// You need to have `docker` and `kind` installed. Have a look at the README at <https://github.com/stackabletech/stackablectl> on how to install them.
        #[clap(short, long)]
        kind_cluster: bool,

        /// Name of the kind cluster created if `--kind-cluster` is specified
        #[clap(
            long,
            default_value = "stackable-data-platform",
            requires = "kind-cluster"
        )]
        kind_cluster_name: String,
    },
}

impl CliCommandStack {
    pub fn handle(&self) {
        match self {
            CliCommandStack::List { output } => list_stacks(output),
            CliCommandStack::Describe { stack, output } => describe_stack(stack, output),
            CliCommandStack::Install {
                stack,
                kind_cluster,
                kind_cluster_name,
            } => {
                kind::handle_cli_arguments(*kind_cluster, kind_cluster_name);
                install_stack(stack);
            }
        }
    }
}

pub fn handle_common_cli_args(args: &CliArgs) {
    let mut stack_files = STACK_FILES.lock().unwrap();
    stack_files.append(&mut args.additional_stack_files.clone());
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Stacks {
    stacks: IndexMap<String, Stack>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Stack {
    description: String,
    stackable_release: String,
    labels: Vec<String>,
    manifests: Vec<StackManifest>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
enum StackManifest {
    #[serde(rename_all = "camelCase")]
    HelmChart {
        release_name: String,
        name: String,
        repo: HelmChartRepo,
        version: String,
        options: serde_yaml::Value,
    },
    PlainYaml(String),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct HelmChartRepo {
    name: String,
    url: String,
}

fn list_stacks(output_type: &OutputType) {
    let output = get_stacks();
    match output_type {
        OutputType::Text => {
            println!("STACK                               STACKABLE RELEASE  DESCRIPTION");
            for (stack_name, stack) in output.stacks.iter() {
                println!(
                    "{:35} {:18} {}",
                    stack_name, stack.stackable_release, stack.description,
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

fn describe_stack(stack_name: &str, output_type: &OutputType) {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Output {
        stack: String,
        description: String,
        stackable_release: String,
        labels: Vec<String>,
    }

    let stack = get_stack(stack_name);
    let output = Output {
        stack: stack_name.to_string(),
        description: stack.description,
        stackable_release: stack.stackable_release,
        labels: stack.labels,
    };

    match output_type {
        OutputType::Text => {
            println!("Stack:              {}", output.stack);
            println!("Description:        {}", output.description);
            println!("Stackable release:  {}", output.stackable_release);
            println!("Labels:             {}", output.labels.join(", "));
        }
        OutputType::Json => {
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        OutputType::Yaml => {
            println!("{}", serde_yaml::to_string(&output).unwrap());
        }
    }
}

fn install_stack(stack_name: &str) {
    info!("Installing stack {stack_name}");
    let stack = get_stack(stack_name);

    release::install_release(&stack.stackable_release, &[], &[]);

    info!("Installing components of stack {stack_name}");
    for manifest in stack.manifests {
        match manifest {
            StackManifest::HelmChart {
                release_name,
                name,
                repo,
                version,
                options,
            } => {
                debug!("Installing helm chart {name} as {release_name}");
                HELM_REPOS
                    .lock()
                    .unwrap()
                    .insert(repo.name.clone(), repo.url);

                let values_yaml = serde_yaml::to_string(&options).unwrap();
                helm::install_helm_release_from_repo(
                    &release_name,
                    &release_name,
                    &repo.name,
                    &name,
                    Some(&version),
                    Some(&values_yaml),
                )
            }
            StackManifest::PlainYaml(yaml_url_or_file) => {
                debug!("Installing yaml manifest from {yaml_url_or_file}");
                match helpers::read_from_url_or_file(&yaml_url_or_file) {
                    Ok(manifests) => kube::deploy_manifest(&manifests),
                    Err(err) => {
                        panic!(
                            "Could not read stack manifests from file \"{}\": {err}",
                            &yaml_url_or_file
                        );
                    }
                }
            }
        }
    }

    info!("Installed stack {stack_name}");
}

/// Cached because of potential slow network calls
#[cached]
fn get_stacks() -> Stacks {
    let mut all_stacks: IndexMap<String, Stack> = IndexMap::new();
    for stack_file in STACK_FILES.lock().unwrap().deref() {
        let yaml = helpers::read_from_url_or_file(stack_file);
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

fn get_stack(stack_name: &str) -> Stack {
    get_stacks()
        .stacks
        .remove(stack_name) // We need to remove to take ownership
        .unwrap_or_else(|| {
            error!("Stack {stack_name} not found. Use `stackablectl stack list` to list the available stacks.");
            exit(1);
        })
}
