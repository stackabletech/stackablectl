use std::error::Error;

use ::kube::{
    api::{DynamicObject, GroupVersionKind, ListParams},
    Api, ResourceExt,
};
use clap::Parser;
use indexmap::IndexMap;
use k8s_openapi::api::{apps::v1::Deployment, core::v1::Secret};
use lazy_static::lazy_static;
use log::warn;
use serde::Serialize;

use crate::{
    arguments::OutputType,
    kube::{self, get_client, get_service_endpoint_urls},
    NAMESPACE,
};

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
    pub extra_infos: Vec<String>,
}

async fn list_services(
    all_namespaces: bool,
    output_type: &OutputType,
) -> Result<(), Box<dyn Error>> {
    let mut output = kube::get_stackable_services(!all_namespaces).await?;
    output.insert(
        "minio".to_string(),
        get_minio_services(!all_namespaces).await?,
    );

    match output_type {
        OutputType::Text => {
            println!("PRODUCT      NAME                                     NAMESPACE                      ENDPOINTS                                          EXTRA INFOS");
            for (product_name, installed_products) in output.iter() {
                for installed_product in installed_products {
                    println!(
                        "{:12} {:40} {:30} {:50} {}",
                        product_name,
                        installed_product.name,
                        installed_product
                            .namespace
                            .as_ref()
                            .map(|s| s.to_string())
                            .unwrap_or_default(),
                        installed_product
                            .endpoints
                            .first()
                            .map(|(name, url)| { format!("{:20} {url}", format!("{name}:")) })
                            .unwrap_or_default(),
                        installed_product
                            .extra_infos
                            .first()
                            .map(|s| s.to_string())
                            .unwrap_or_default(),
                    );

                    let mut endpoints = installed_product.endpoints.iter().skip(1);
                    let mut extra_infos = installed_product.extra_infos.iter().skip(1);

                    loop {
                        let endpoint = endpoints.next();
                        let extra_info = extra_infos.next();

                        println!(
                            "                                                                                     {:50} {}",
                            endpoint
                                .map(|(name, url)| { format!("{:20} {url}", format!("{name}:")) })
                                .unwrap_or_default(),
                            extra_info.map(|s| s.to_string()).unwrap_or_default(),
                        );

                        if endpoint.is_none() && extra_info.is_none() {
                            break;
                        }
                    }
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
        "druid" => vec![
            format!("{product_name}-router"),
            format!("{product_name}-coordinator"),
        ],
        "hive" => vec![],
        "superset" => vec![format!("{product_name}-external")],
        "trino" => vec![format!("{product_name}-coordinator")],
        "zookeeper" => vec![product_name.to_string()],
        _ => {
            warn!("Cannot calculated exposed services names as product {product} is not known");
            vec![]
        }
    }
}

pub async fn get_extra_infos(
    product: &str,
    product_crd: &DynamicObject,
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut result = match product_crd.data["spec"]["version"].as_str() {
        Some(version) => Vec::from([format!("Version {version}")]),
        None => Vec::new(),
    };

    match product {
        "superset" => {
            if let Some(secret_name) = product_crd.data["spec"]["credentialsSecret"].as_str() {
                let client = get_client().await?;
                let secret_api: Api<Secret> =
                    Api::namespaced(client, &product_crd.namespace().unwrap());
                let secret = secret_api.get(secret_name).await?;
                let secret_data = secret.data.unwrap();

                if let (Some(username), Some(password)) = (
                    secret_data.get("adminUser.username"),
                    secret_data.get("adminUser.password"),
                ) {
                    let username = String::from_utf8(username.0.clone()).unwrap();
                    let password = String::from_utf8(password.0.clone()).unwrap();
                    result.push(format!("user: {username}, password: {password}"));
                }
            }
        }
        _ => (),
    }

    Ok(result)
}

async fn get_minio_services(namespaced: bool) -> Result<Vec<InstalledProduct>, Box<dyn Error>> {
    let client = get_client().await?;
    let deployment_api: Api<Deployment> = match namespaced {
        true => Api::namespaced(client.clone(), NAMESPACE.lock().unwrap().as_str()),
        false => Api::all(client.clone()),
    };
    let list_params = ListParams::default().labels("app=minio");
    let minio_deployments = deployment_api.list(&list_params).await?;

    let mut result = Vec::new();
    for minio_deployment in minio_deployments {
        let deployment_name = minio_deployment.name();
        let deployment_namespace = minio_deployment.namespace().unwrap();

        let service_names = vec![
            deployment_name.clone(),
            format!("{deployment_name}-console"),
        ];
        let extra_infos = vec![
            "This service is not part of the official Stackable Platform".to_string(),
            "It is provided as a helper utility".to_string(),
        ];

        let mut endpoints = IndexMap::new();
        for service_name in service_names {
            let service_endpoint_urls = get_service_endpoint_urls(
                &service_name,
                &deployment_name,
                &deployment_namespace,
                client.clone(),
            )
            .await;
            match service_endpoint_urls {
                Ok(service_endpoint_urls) => endpoints.extend(service_endpoint_urls),
                Err(err) => warn!("Failed to get endpoint_urls of service {service_name}: {err}"),
            }
        }
        let product = InstalledProduct {
            name: deployment_name,
            namespace: Some(deployment_namespace),
            endpoints,
            extra_infos,
        };
        result.push(product);
    }

    Ok(result)
}
