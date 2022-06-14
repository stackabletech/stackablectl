use core::panic;
use std::error::Error;

use ::kube::api::GroupVersionKind;
use clap::Parser;
use indexmap::IndexMap;
use lazy_static::lazy_static;
use serde::Serialize;

use crate::{arguments::OutputType, kube};

// Additional services we need to think of in the future
// * MinIO
lazy_static! {
    pub static ref PRODUCT_CRDS: IndexMap<&'static str, GroupVersionKind> = IndexMap::from([
        (
            "airflow",
            GroupVersionKind {
                group: "airflow.stackable.tech".to_string(),
                version: "v1alpha1".to_string(),
                kind: "AirflowCluster".to_string(),
            }
        ),
        (
            "druid",
            GroupVersionKind {
                group: "druid.stackable.tech".to_string(),
                version: "v1alpha1".to_string(),
                kind: "DruidCluster".to_string(),
            }
        ),
        (
            "hbase",
            GroupVersionKind {
                group: "hbase.stackable.tech".to_string(),
                version: "v1alpha1".to_string(),
                kind: "HbaseCluster".to_string(),
            }
        ),
        (
            "hdfs",
            GroupVersionKind {
                group: "hdfs.stackable.tech".to_string(),
                version: "v1alpha1".to_string(),
                kind: "HdfsCluster".to_string(),
            }
        ),
        (
            "hive",
            GroupVersionKind {
                group: "hive.stackable.tech".to_string(),
                version: "v1alpha1".to_string(),
                kind: "HiveCluster".to_string(),
            }
        ),
        (
            "kafka",
            GroupVersionKind {
                group: "kafka.stackable.tech".to_string(),
                version: "v1alpha1".to_string(),
                kind: "KafkaCluster".to_string(),
            }
        ),
        (
            "nifi",
            GroupVersionKind {
                group: "nifi.stackable.tech".to_string(),
                version: "v1alpha1".to_string(),
                kind: "NifiCluster".to_string(),
            }
        ),
        (
            "opa",
            GroupVersionKind {
                group: "opa.stackable.tech".to_string(),
                version: "v1alpha1".to_string(),
                kind: "OpenPolicyAgent".to_string(),
            }
        ),
        (
            "spark",
            GroupVersionKind {
                group: "spark.stackable.tech".to_string(),
                version: "v1alpha1".to_string(),
                kind: "SparkCluster".to_string(),
            }
        ),
        (
            "superset",
            GroupVersionKind {
                group: "superset.stackable.tech".to_string(),
                version: "v1alpha1".to_string(),
                kind: "SupersetCluster".to_string(),
            }
        ),
        (
            "trino",
            GroupVersionKind {
                group: "trino.stackable.tech".to_string(),
                version: "v1alpha1".to_string(),
                kind: "TrinoCluster".to_string(),
            }
        ),
        (
            "zookeeper",
            GroupVersionKind {
                group: "zookeeper.stackable.tech".to_string(),
                version: "v1alpha1".to_string(),
                kind: "ZookeeperCluster".to_string(),
            }
        ),
    ]);
}

#[derive(Parser)]
pub enum CliCommandServices {
    /// List deployed services
    #[clap(alias("ls"))]
    List {
        /// If specified services of all namespaces will be shown, not only the namespace you're currently in
        #[clap(short, long)]
        all_namespaces: bool,

        #[clap(short, long, arg_enum, default_value = "text")]
        output: OutputType,
    },
}

impl CliCommandServices {
    pub async fn handle(&self) -> Result<(), Box<dyn Error>> {
        match self {
            CliCommandServices::List {
                all_namespaces,
                output,
            } => list_services(*all_namespaces, output).await?,
        }
        Ok(())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledProduct {
    pub name: String,
    pub namespace: Option<String>, // Some CRDs are cluster scoped
    pub endpoints: IndexMap<String, String>, // key: service name (e.g. web-ui), value: url
}

async fn list_services(
    all_namespaces: bool,
    output_type: &OutputType,
) -> Result<(), Box<dyn Error>> {
    let output = kube::get_services(!all_namespaces).await?;

    match output_type {
        OutputType::Text => {
            println!("PRODUCT         NAMESPACE                      NAME                                     ENDPOINTS");
            for (product_name, installed_products) in output.iter() {
                for installed_product in installed_products {
                    println!(
                        "{:15} {:30} {:40} {}",
                        product_name,
                        installed_product
                            .namespace
                            .as_ref()
                            .unwrap_or(&"~".to_string()),
                        installed_product.name,
                        installed_product.endpoints.iter().map(|(name, url)| {
                            format!("{:10} {url}", format!("{name}:"))
                        }).collect::<Vec<_>>().join("\n                                                                                             ")
                    );
                }
            }
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

pub fn get_service_names(product_name: &str, product: &str) -> Vec<String> {
    match product {
        "druid" => vec![format!("{product_name}-router")],
        "superset" => vec![format!("{product_name}-external")],
        "zookeeper" => vec![],
        _ => panic!("product {product} not known"),
    }
}
