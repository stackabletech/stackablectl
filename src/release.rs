use crate::{arguments::OutputType, helpers, kind, operator, operator::Operator, CliArgs};
use cached::proc_macro::cached;
use clap::{ArgGroup, Parser, ValueHint};
use indexmap::IndexMap;
use lazy_static::lazy_static;
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::{ops::Deref, process::exit, sync::Mutex};

lazy_static! {
    pub static ref RELEASE_FILES: Mutex<Vec<String>> = Mutex::new(vec![
        "https://raw.githubusercontent.com/stackabletech/release/main/releases.yaml".to_string()
    ]);
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
        /// Name of the release to describe
        #[clap(required = true, value_hint = ValueHint::Other)]
        release: String,

        #[clap(short, long, arg_enum, default_value = "text")]
        output: OutputType,
    },
    /// Install a specific release
    #[clap(alias("in"))]
    #[clap(group(
        ArgGroup::new("list-of-products")
            .required(false)
            .args(&["include-products", "exclude-products"]),
    ))]
    Install {
        /// Name of the release to install
        #[clap(required = true, value_hint = ValueHint::Other)]
        release: String,

        /// Whitelist of product operators to install.
        /// Mutually exclusive with `--exclude-products`
        #[clap(short, long, value_hint = ValueHint::Other)]
        include_products: Vec<String>,

        /// Blacklist of product operators to install.
        /// Mutually exclusive with `--include-products`
        #[clap(short, long, value_hint = ValueHint::Other)]
        exclude_products: Vec<String>,

        /// If specified a local kubernetes cluster consisting of 4 nodes for testing purposes will be created.
        /// Kind is a tool to spin up a local kubernetes cluster running on docker on your machine.
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
    /// Uninstall a release
    #[clap(alias("un"))]
    Uninstall {
        /// Name of the release to uninstall
        #[clap(required = true, value_hint = ValueHint::Other)]
        release: String,
    },
}

impl CliCommandRelease {
    pub async fn handle(&self) {
        match self {
            CliCommandRelease::List { output } => list_releases(output).await,
            CliCommandRelease::Describe { release, output } => {
                describe_release(release, output).await
            }
            CliCommandRelease::Install {
                release,
                include_products,
                exclude_products,
                kind_cluster,
                kind_cluster_name,
            } => {
                kind::handle_cli_arguments(*kind_cluster, kind_cluster_name);
                install_release(release, include_products, exclude_products).await;
            }
            CliCommandRelease::Uninstall { release } => uninstall_release(release).await,
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

async fn list_releases(output_type: &OutputType) {
    let output = get_releases().await;
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

async fn describe_release(release_name: &str, output_type: &OutputType) {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Output {
        release: String,
        release_date: String,
        description: String,
        products: IndexMap<String, ReleaseProduct>,
    }

    let release = get_release(release_name).await;
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

/// If include_operators is an non-empty list only the whitelisted product operators will be installed.
/// If exclude_operators is an non-empty list the blacklisted product operators will be skipped.
pub async fn install_release(
    release_name: &str,
    include_products: &[String],
    exclude_products: &[String],
) {
    info!("Installing release {release_name}");
    let release = get_release(release_name).await;

    for (product_name, product) in release.products.into_iter() {
        let included = include_products.is_empty() || include_products.contains(&product_name);
        let excluded = exclude_products.contains(&product_name);

        if included && !excluded {
            Operator::new(product_name, Some(product.operator_version))
                .expect("Failed to construct operator definition")
                .install();
        }
    }
}

async fn uninstall_release(release_name: &str) {
    info!("Uninstalling release {release_name}");
    let release = get_release(release_name).await;

    operator::uninstall_operators(&release.products.into_keys().collect());
}

/// Cached because of potential slow network calls
#[cached]
async fn get_releases() -> Releases {
    let mut all_releases: IndexMap<String, Release> = IndexMap::new();
    let release_files = RELEASE_FILES.lock().unwrap().deref().clone();
    for release_file in release_files {
        let yaml = helpers::read_from_url_or_file(&release_file).await;
        match yaml {
            Ok(yaml) => match serde_yaml::from_str::<Releases>(&yaml) {
                Ok(releases) => all_releases.extend(releases.releases),
                Err(err) => warn!("Failed to parse release list from {release_file}: {err}"),
            },
            Err(err) => {
                warn!("Could not read from releases file \"{release_file}\": {err}");
            }
        }
    }

    Releases {
        releases: all_releases,
    }
}

async fn get_release(release_name: &str) -> Release {
    get_releases()
    .await
        .releases
        .remove(release_name) // We need to remove to take ownership
        .unwrap_or_else(|| {
            error!("Release {release_name} not found. Use `stackablectl release list` to list the available releases.");
            exit(1);
        })
}
