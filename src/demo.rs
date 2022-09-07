use crate::{
    arguments::OutputType,
    helpers, kind,
    stack::{self, StackManifest},
    CliArgs,
};
use cached::proc_macro::cached;
use clap::{Parser, ValueHint};
use indexmap::IndexMap;
use lazy_static::lazy_static;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::{error::Error, ops::Deref, sync::Mutex};

lazy_static! {
    pub static ref DEMO_FILES: Mutex<Vec<String>> = Mutex::new(vec![
        "https://raw.githubusercontent.com/stackabletech/stackablectl/main/demos/demos-v1.yaml"
            .to_string(),
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
        #[clap(required = true, value_hint = ValueHint::Other)]
        demo: String,

        #[clap(short, long, arg_enum, default_value = "text")]
        output: OutputType,
    },
    /// Install a specific demo
    #[clap(alias("in"))]
    Install {
        /// Name of the demo to install
        #[clap(required = true, value_hint = ValueHint::Other)]
        demo: String,

        /// If specified, a local Kubernetes cluster consisting of 4 nodes (1 for control-plane and 3 workers) for testing purposes will be created.
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
}

impl CliCommandDemo {
    pub async fn handle(&self) -> Result<(), Box<dyn Error>> {
        match self {
            CliCommandDemo::List { output } => list_demos(output).await?,
            CliCommandDemo::Describe { demo, output } => describe_demo(demo, output).await?,
            CliCommandDemo::Install {
                demo,
                kind_cluster,
                kind_cluster_name,
            } => {
                kind::handle_cli_arguments(*kind_cluster, kind_cluster_name)?;
                install_demo(demo).await?;
            }
        }
        Ok(())
    }
}

pub fn handle_common_cli_args(args: &CliArgs) {
    let mut demo_files = DEMO_FILES.lock().unwrap();
    demo_files.extend_from_slice(&args.additional_demos_file);
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Demos {
    #[serde(with = "serde_yaml::with::singleton_map_recursive")]
    demos: IndexMap<String, Demo>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Demo {
    description: String,
    documentation: Option<String>,
    stackable_stack: String,
    labels: Vec<String>,
    manifests: Vec<StackManifest>,
}

async fn list_demos(output_type: &OutputType) -> Result<(), Box<dyn Error>> {
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
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputType::Yaml => {
            println!("{}", serde_yaml::to_string(&output)?);
        }
    }

    Ok(())
}

async fn describe_demo(demo_name: &str, output_type: &OutputType) -> Result<(), Box<dyn Error>> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Output {
        demo: String,
        description: String,
        documentation: Option<String>,
        stackable_stack: String,
        labels: Vec<String>,
    }

    let demo = get_demo(demo_name).await?;
    let output = Output {
        demo: demo_name.to_string(),
        description: demo.description,
        documentation: demo.documentation,
        stackable_stack: demo.stackable_stack,
        labels: demo.labels,
    };

    match output_type {
        OutputType::Text => {
            println!("Demo:               {}", output.demo);
            println!("Description:        {}", output.description);
            if let Some(documentation) = output.documentation {
                println!("Documentation:      {}", documentation);
            }
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

    Ok(())
}

async fn install_demo(demo_name: &str) -> Result<(), Box<dyn Error>> {
    info!("Installing demo {demo_name}");
    let demo = get_demo(demo_name).await?;
    stack::install_stack(&demo.stackable_stack).await?;
    info!("Installing components of demo {demo_name}");
    stack::install_manifests(&demo.manifests).await?;

    info!("Installed demo {demo_name}. Use \"stackablectl services list\" to list the installed services");
    Ok(())
}

/// Cached because of potential slow network calls
#[cached]
async fn get_demos() -> Demos {
    let mut all_demos = IndexMap::new();
    let demo_files = DEMO_FILES.lock().unwrap().deref().clone();
    for demo_file in demo_files {
        let yaml = helpers::read_from_url_or_file(&demo_file).await;
        match yaml {
            Ok(yaml) => match serde_yaml::from_str::<Demos>(&yaml) {
                Ok(demos) => all_demos.extend(demos.demos),
                Err(err) => warn!("Failed to parse demo list from {demo_file}: {err}"),
            },
            Err(err) => {
                warn!("Could not read from demo file \"{demo_file}\": {err}");
            }
        }
    }

    Demos { demos: all_demos }
}

async fn get_demo(demo_name: &str) -> Result<Demo, Box<dyn Error>> {
    get_demos()
    .await
        .demos
        .remove(demo_name) // We need to remove to take ownership
        .ok_or_else(|| format!("Demo {demo_name} not found. Use `stackablectl demo list` to list the available demos.").into())
}
