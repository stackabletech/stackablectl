use crate::{
    arguments::{CliArgs, OutputType},
    helpers, kind,
    stack::{self, StackManifest},
};
use cached::proc_macro::cached;
use clap::Parser;
use indexmap::IndexMap;
use lazy_static::{__Deref, lazy_static};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::{error::Error, process::exit, sync::Mutex};

lazy_static! {
    pub static ref DEMO_FILES: Mutex<Vec<String>> = Mutex::new(vec![
        "https://raw.githubusercontent.com/stackabletech/stackablectl/main/demos.yaml".to_string(),
    ]);
}

#[derive(Parser)]
pub enum CliCommandDemo {
    /// List all the available demos
    #[clap(alias("ls"))]
    List {
        #[clap(short, long, arg_enum, default_value = "text")]
        output: OutputType,
    },
    /// Show details of a specific demo
    #[clap(alias("desc"))]
    Describe {
        /// Name of the demo to describe
        #[clap(required = true)]
        demo: String,

        #[clap(short, long, arg_enum, default_value = "text")]
        output: OutputType,
    },
    /// Install a specific demo
    #[clap(alias("in"))]
    Install {
        /// Name of the demo to install
        #[clap(required = true)]
        demo: String,

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

impl CliCommandDemo {
    pub async fn handle(&self) -> Result<(), Box<dyn Error>> {
        match self {
            CliCommandDemo::List { output } => list_demos(output).await,
            CliCommandDemo::Describe { demo, output } => describe_demo(demo, output).await,
            CliCommandDemo::Install {
                demo,
                kind_cluster,
                kind_cluster_name,
            } => {
                kind::handle_cli_arguments(*kind_cluster, kind_cluster_name);
                install_demo(demo).await?;
            }
        }
        Ok(())
    }
}

pub fn handle_common_cli_args(args: &CliArgs) {
    let mut demo_files = DEMO_FILES.lock().unwrap();
    demo_files.append(&mut args.additional_demo_files.clone());
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Demos {
    demos: IndexMap<String, Demo>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Demo {
    description: String,
    stackable_stack: String,
    labels: Vec<String>,
    manifests: Vec<StackManifest>,
}

async fn list_demos(output_type: &OutputType) {
    let output = get_demos().await;
    match output_type {
        OutputType::Text => {
            println!("DEMO                                STACKABLE STACK           DESCRIPTION");
            for (demo_name, demo) in output.demos.iter() {
                println!(
                    "{:35} {:25} {}",
                    demo_name, demo.stackable_stack, demo.description,
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

async fn describe_demo(demo_name: &str, output_type: &OutputType) {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Output {
        demo: String,
        description: String,
        stackable_stack: String,
        labels: Vec<String>,
    }

    let demo = get_demo(demo_name).await;
    let output = Output {
        demo: demo_name.to_string(),
        description: demo.description,
        stackable_stack: demo.stackable_stack,
        labels: demo.labels,
    };

    match output_type {
        OutputType::Text => {
            println!("Demo:               {}", output.demo);
            println!("Description:        {}", output.description);
            println!("Stackable stack:    {}", output.stackable_stack);
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

async fn install_demo(demo_name: &str) -> Result<(), Box<dyn Error>> {
    info!("Installing demo {demo_name}");
    let demo = get_demo(demo_name).await;

    stack::install_stack(&demo.stackable_stack).await?;

    info!("Installing components of demo {demo_name}");
    stack::install_manifests(&demo.manifests).await?;

    info!("Installed demo {demo_name}. Use \"stackablectl services list\" to list the installed services");
    Ok(())
}

/// Cached because of potential slow network calls
#[cached]
async fn get_demos() -> Demos {
    let mut add_demos: IndexMap<String, Demo> = IndexMap::new();
    let demo_files = DEMO_FILES.lock().unwrap().deref().clone();
    for demo_file in demo_files {
        let yaml = helpers::read_from_url_or_file(&demo_file).await;
        match yaml {
            Ok(yaml) => match serde_yaml::from_str::<Demos>(&yaml) {
                Ok(demos) => add_demos.extend(demos.demos),
                Err(err) => warn!("Failed to parse demo list from {demo_file}: {err}"),
            },
            Err(err) => {
                warn!("Could not read from demos file \"{demo_file}\": {err}");
            }
        }
    }

    Demos { demos: add_demos }
}

async fn get_demo(demo_name: &str) -> Demo {
    get_demos()
    .await
        .demos
        .remove(demo_name) // We need to remove to take ownership
        .unwrap_or_else(|| {
            error!("Demo {demo_name} not found. Use `stackablectl demo list` to list the available demos.");
            exit(1);
        })
}
