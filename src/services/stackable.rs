use std::error::Error;

use indexmap::IndexMap;
use k8s_openapi::api::core::v1::Service;
use kube::{
    api::ListParams,
    core::{DynamicObject, GroupVersionKind},
    Api, Discovery, ResourceExt,
};
use lazy_static::lazy_static;
use log::{debug, warn};

use crate::{
    kube::{get_client, get_service_endpoint_urls},
    NAMESPACE,
};

use super::{get_credentials_from_secret, InstalledProduct};

lazy_static! {
    pub static ref STACKABLE_PRODUCT_CRDS: IndexMap<&'static str, GroupVersionKind> =
        IndexMap::from([
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
                    kind: "OpaCluster".to_string(),
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

pub async fn get_stackable_services(
    namespaced: bool,
    redact_credentials: bool,
    show_versions: bool,
) -> Result<IndexMap<String, Vec<InstalledProduct>>, Box<dyn Error>> {
    let mut result = IndexMap::new();
    let namespace = NAMESPACE.lock()?.clone();

    let client = get_client().await?;
    let discovery = Discovery::new(client.clone()).run().await?;

    for (product_name, product_gvk) in STACKABLE_PRODUCT_CRDS.iter() {
        let object_api_resource = match discovery.resolve_gvk(product_gvk) {
            Some((object_api_resource, _)) => object_api_resource,
            None => {
                debug!("Failed to list services of product {product_name} because the gvk {product_gvk:?} can not be resolved");
                continue;
            }
        };

        let object_api: Api<DynamicObject> = match namespaced {
            true => Api::namespaced_with(client.clone(), &namespace, &object_api_resource),
            false => Api::all_with(client.clone(), &object_api_resource),
        };

        let objects = object_api.list(&ListParams::default()).await?;
        let mut installed_products = Vec::new();
        for object in objects {
            let object_name = object.name_any();
            let object_namespace = match object.namespace() {
                Some(namespace) => namespace,
                // If the custom resource does not have a namespace set it can't expose a service
                None => continue,
            };

            let service_api: Api<Service> =
                Api::namespaced(client.clone(), object_namespace.as_str());
            let service_list_params = ListParams::default()
                .labels(format!("app.kubernetes.io/name={product_name}").as_str())
                .labels(format!("app.kubernetes.io/instance={object_name}").as_str());
            let services = service_api.list(&service_list_params).await?;

            let extra_infos =
                get_extra_infos(product_name, &object, redact_credentials, show_versions).await?;

            let mut endpoints = IndexMap::new();
            for service in services {
                let service_endpoint_urls =
                    get_service_endpoint_urls(&service, &object_name, client.clone()).await;
                match service_endpoint_urls {
                    Ok(service_endpoint_urls) => endpoints.extend(service_endpoint_urls),
                    Err(err) => warn!(
                        "Failed to get endpoint_urls of service {service_name}: {err}",
                        service_name = service.name_unchecked(),
                    ),
                }
            }
            let product = InstalledProduct {
                name: object_name,
                namespace: Some(object_namespace),
                endpoints,
                extra_infos,
            };
            installed_products.push(product);
        }
        result.insert(product_name.to_string(), installed_products);
    }

    Ok(result)
}

pub async fn get_extra_infos(
    product: &str,
    product_crd: &DynamicObject,
    redact_credentials: bool,
    show_versions: bool,
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut result = Vec::new();

    match product {
        "airflow" | "superset" => {
            if let Some(secret_name) = product_crd.data["spec"]["credentialsSecret"].as_str() {
                let credentials = get_credentials_from_secret(
                    secret_name,
                    product_crd
                        .namespace()
                        .ok_or(format!(
                            "The custom resource {product_crd:?} had no namespace set"
                        ))?
                        .as_str(),
                    "adminUser.username",
                    "adminUser.password",
                    redact_credentials,
                )
                .await?;

                if let Some((username, password)) = credentials {
                    result.push(format!("Admin user: {username}, password: {password}"));
                }
            }
        }
        "nifi" => {
            if let Some(admin_credentials_secret) = product_crd.data["spec"]["config"]
                ["authentication"]["method"]["singleUser"]["adminCredentialsSecret"]
                .as_str()
            {
                let credentials = get_credentials_from_secret(
                    admin_credentials_secret,
                    product_crd
                        .namespace()
                        .ok_or(format!(
                            "The custom resource {product_crd:?} had no namespace set"
                        ))?
                        .as_str(),
                    "username",
                    "password",
                    redact_credentials,
                )
                .await?;

                if let Some((username, password)) = credentials {
                    result.push(format!("Admin user: {username}, password: {password}"));
                }
            }
        }
        _ => (),
    }

    if show_versions {
        if let Some(version) = product_crd.data["spec"]["version"].as_str() {
            result.push(format!("version {version}"));
        }
    }

    Ok(result)
}
