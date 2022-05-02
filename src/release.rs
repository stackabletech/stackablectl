use crate::arguments::OutputType;
use crate::operator::Operator;
use crate::{helpers, kind, operator, CliArgs};
use cached::proc_macro::cached;
use clap::Parser;
use indexmap::IndexMap;
use lazy_static::lazy_static;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::process::exit;
use std::sync::Mutex;

lazy_static! {
    pub static ref RELEASE_FILES: Mutex<Vec<String>> =
        Mutex::new(vec!["releases.yaml".to_string()]);
}

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
    pub fn handle(&self) {
        match self {
            CliCommandRelease::List { output } => list_releases(output),
            CliCommandRelease::Describe { release, output } => describe_release(release, output),
            CliCommandRelease::Install {
                release,
                kind_cluster,
            } => {
                kind::handle_cli_arguments(kind_cluster);
                install_release(release);
            }
            CliCommandRelease::Uninstall { release } => uninstall_release(release),
        }
    }
}

pub fn handle_common_cli_args(args: &CliArgs) {
    let mut release_files = RELEASE_FILES.lock().unwrap();
    release_files.append(&mut args.additional_release_files.clone());
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Releases {
    releases: IndexMap<String, Release>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Release {
    release_date: String,
    description: String,
    products: IndexMap<String, ReleaseProduct>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
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

fn install_release(release_name: &str) {
    info!("Installing release {release_name}");
    let release = get_release(release_name);

    for (product_name, product) in release.products.into_iter() {
        Operator::new(product_name, Some(product.operator_version))
            .expect("Failed to construct operator definition")
            .install();
    }
}

fn uninstall_release(release_name: &str) {
    info!("Uninstalling release {release_name}");
    let release = get_release(release_name);

    operator::uninstall_operators(&release.products.into_keys().collect());
}

/// Cached because of potential slow network calls
#[cached]
fn get_releases() -> Releases {
    let mut all_releases: IndexMap<String, Release> = IndexMap::new();
    for release_file in RELEASE_FILES.lock().unwrap().deref() {
        let yaml = helpers::read_from_url_or_file(release_file);
        match yaml {
            Ok(yaml) => {
                let releases: Releases = serde_yaml::from_str(&yaml)
                    .unwrap_or_else(|err| panic!("Failed to parse release list from {release_file}: {err}"));
                all_releases.extend(releases.releases.clone());
            }
            Err(err) => {
                warn!("Could not read from releases file \"{release_file}\": {err}");
            }
        }
    }

    Releases {
        releases: all_releases,
    }
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
