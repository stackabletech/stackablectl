use crate::arguments::OutputType;
use crate::operator::Operator;
use crate::{kind, operator};
use clap::Parser;
use indexmap::IndexMap;
use log::{error, info, LevelFilter};
use serde::{Deserialize, Serialize};
use std::process::exit;

#[derive(Parser)]
pub enum CliCommandRelease {
    /// List all the available releases
    #[clap(alias("ls"))]
    List {
        #[clap(short, long, arg_enum, default_value = "text")]
        output: OutputType,
    },
    /// Show details of a specific release
    #[clap(alias("desc"))]
    Describe {
        release: String,
        #[clap(short, long, arg_enum, default_value = "text")]
        output: OutputType,
    },
    /// Install a specific release
    #[clap(alias("in"))]
    Install {
        /// Name of the release to install
        #[clap(required = true)]
        release: String,

        /// If this argument is specified a local kubernetes cluster for testing purposes is created.
        /// Kind is a tool to spin up a local kubernetes cluster running on docker on your machine.
        /// This scripts creates such a cluster consisting of 4 nodes to test the Stackable Data Platform.
        /// The default cluster name is `stackable-data-platform` which can be overwritten by specifying the cluster name after `--kind-cluster`
        /// You need to have `docker` and `kind` installed. Have a look at the README at https://github.com/stackabletech/stackablectl on how to install them
        #[clap(short, long)]
        kind_cluster: Option<Option<String>>,
    },
    /// Uninstall a release
    #[clap(alias("un"))]
    Uninstall {
        /// Name of the release to uninstall
        #[clap(required = true)]
        release: String,
    },
}

impl CliCommandRelease {
    pub fn handle(&self, log_level: LevelFilter) {
        match self {
            CliCommandRelease::List { output } => list_releases(output),
            CliCommandRelease::Describe { release, output } => describe_release(release, output),
            CliCommandRelease::Install {
                release,
                kind_cluster,
            } => {
                kind::handle_cli_arguments(kind_cluster);
                install_release(release, log_level);
            }
            CliCommandRelease::Uninstall { release } => uninstall_release(release, log_level),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Releases {
    releases: IndexMap<String, Release>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Release {
    release_date: String,
    description: String,
    products: IndexMap<String, ReleaseProduct>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ReleaseProduct {
    operator_version: String,
}

fn list_releases(output_type: &OutputType) {
    let output = get_releases();
    match output_type {
        OutputType::Text => {
            println!("RELEASE            RELEASE DATE   DESCRIPTION");
            for (release_name, release_entry) in output.releases.iter() {
                println!(
                    "{:18} {:14} {}",
                    release_name, release_entry.release_date, release_entry.description,
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

fn describe_release(release_name: &str, output_type: &OutputType) {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Output {
        release: String,
        release_date: String,
        description: String,
        products: IndexMap<String, ReleaseProduct>,
    }

    let release = get_release(release_name);
    let output = Output {
        release: release_name.to_string(),
        release_date: release.release_date,
        description: release.description,
        products: release.products,
    };

    match output_type {
        OutputType::Text => {
            println!("Release:            {}", output.release);
            println!("Release date:       {}", output.release_date);
            println!("Description:        {}", output.description);
            println!("Included products:");
            println!();
            println!("PRODUCT             OPERATOR VERSION");
            for (product_name, product) in output.products.iter() {
                println!("{:19} {}", product_name, product.operator_version);
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

fn install_release(release_name: &str, log_level: LevelFilter) {
    info!("Installing release {release_name}");
    let release = get_release(release_name);

    for (product_name, product) in release.products.into_iter() {
        Operator::new(product_name, Some(product.operator_version))
            .expect("Failed to construct operator definition")
            .install(log_level);
    }
}

fn uninstall_release(release_name: &str, log_level: LevelFilter) {
    info!("Uninstalling release {release_name}");
    let release = get_release(release_name);

    operator::uninstall_operators(&release.products.into_keys().collect(), log_level);
}

fn get_releases() -> Releases {
    // TODO Read from URL/file/whatever
    let file_name = "releases.yaml";
    let file = std::fs::File::open(file_name)
        .expect(format!("Could not read releases from {file_name}").as_str());

    serde_yaml::from_reader(file)
        .expect(format!("Failed to parse release list from {file_name}").as_str())
}

fn get_release(release_name: &str) -> Release {
    get_releases()
        .releases
        .remove(release_name) // We need to remove to take ownership
        .unwrap_or_else(|| {
            error!("Release {release_name} not found. Use `stackablectl release list` to list the available releases.");
            exit(1);
        })
}
