use clap::Parser;
use cli_table::{
    format::{Border, HorizontalLine, Separator},
    Cell, Table,
};
use indexmap::IndexMap;
use k8s_openapi::api::{
    apps::v1::Deployment,
    core::v1::{Secret, Service},
};
use kube::{
    api::{DynamicObject, GroupVersionKind, ListParams},
    Api, Discovery, ResourceExt,
};
use lazy_static::lazy_static;
use log::{debug, warn};
use serde::Serialize;
use std::{error::Error, vec};

use crate::{
    arguments::OutputType,
    kube::{get_client, get_service_endpoint_urls},
    NAMESPACE,
};

pub static REDACTED_PASSWORD: &str = "<redacted>";

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

#[derive(Parser)]
pub enum CliCommandServices {
    /// List deployed services
    #[clap(alias("ls"))]
    List {
        /// If specified services of all namespaces will be shown, not only the namespace you're currently in
        #[clap(short, long)]
        all_namespaces: bool,

        /// Don't show credentials in the output
        #[clap(short, long)]
        redact_credentials: bool,

        /// Show the product versions in the output
        #[clap(long)]
        show_versions: bool,

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
                redact_credentials,
                show_versions,
            } => {
                list_services(*all_namespaces, *redact_credentials, *show_versions, output).await?;
            }
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
    redact_credentials: bool,
    show_versions: bool,
    output_type: &OutputType,
) -> Result<(), Box<dyn Error>> {
    let mut output =
        get_stackable_services(!all_namespaces, redact_credentials, show_versions).await?;
    output.insert(
        "minio".to_string(),
        get_minio_services(!all_namespaces, redact_credentials).await?,
    );

    match output_type {
        OutputType::Text => {
            let mut table = vec![];

            let max_endpoint_name_length = output
                .values()
                .flatten()
                .flat_map(|p| &p.endpoints)
                .map(|e| e.0.len())
                .max()
                .unwrap_or_default();

            for (product_name, installed_products) in output {
                for installed_product in installed_products {
                    let mut endpoints = vec![];
                    for endpoint in &installed_product.endpoints {
                        endpoints.push(vec![endpoint.0.as_str(), endpoint.1.as_str()]);
                    }

                    let endpoints = installed_product
                        .endpoints
                        .iter()
                        .map(|(name, url)| {
                            format!("{name:width$}{url}", width = max_endpoint_name_length + 1)
                        })
                        .collect::<Vec<_>>()
                        .join("\n");

                    table.push(vec![
                        (&product_name).cell(),
                        installed_product.name.as_str().cell(),
                        installed_product
                            .namespace
                            .clone()
                            .unwrap_or_default()
                            .cell(),
                        endpoints.cell(),
                        installed_product.extra_infos.join("\n").cell(),
                    ]);
                }
            }
            let table = table
                .table()
                .title(vec![
                    "PRODUCT".cell(),
                    "NAME".cell(),
                    "NAMESPACE".cell(),
                    "ENDPOINTS".cell(),
                    "EXTRA INFOS".cell(),
                ])
                .border(Border::builder().build())
                .separator(
                    Separator::builder()
                        .row(Some(HorizontalLine::new(' ', ' ', ' ', ' ')))
                        .build(),
                );

            print!("{}", table.display()?);
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

async fn get_credentials_from_secret(
    secret_name: &str,
    secret_namespace: &str,
    username_key: &str,
    password_key: &str,
    redact_credentials: bool,
) -> Result<Option<(String, String)>, Box<dyn Error>> {
    let client = get_client().await?;
    let secret_api: Api<Secret> = Api::namespaced(client, secret_namespace);

    let secret = secret_api.get(secret_name).await?;
    let secret_data = secret
        .data
        .ok_or(format!("Secret {secret_name} had no data"))?;

    match (secret_data.get(username_key), secret_data.get(password_key)) {
        (Some(username), Some(password)) => {
            let username = String::from_utf8(username.0.clone())?;
            let password = if redact_credentials {
                REDACTED_PASSWORD.to_string()
            } else {
                String::from_utf8(password.0.clone())?
            };
            Ok(Some((username, password)))
        }
        _ => Ok(None),
    }
}

async fn get_minio_services(
    namespaced: bool,
    redact_credentials: bool,
) -> Result<Vec<InstalledProduct>, Box<dyn Error>> {
    let client = get_client().await?;
    let deployment_api: Api<Deployment> = match namespaced {
        true => Api::namespaced(client.clone(), NAMESPACE.lock()?.as_str()),
        false => Api::all(client.clone()),
    };
    let list_params = ListParams::default().labels("app=minio");
    let minio_deployments = deployment_api.list(&list_params).await?;

    let mut result = Vec::new();
    for minio_deployment in minio_deployments {
        let deployment_name = minio_deployment.name_unchecked();
        let deployment_namespace = minio_deployment.namespace().ok_or(format!(
            "MinIO deployment {deployment_name} had no namespace"
        ))?;

        let service_api = Api::namespaced(client.clone(), &deployment_namespace);
        let service_names = vec![
            deployment_name.clone(),
            format!("{deployment_name}-console"),
        ];

        let mut endpoints = IndexMap::new();
        for service_name in service_names {
            let service = service_api.get(&service_name).await?;
            let service_endpoint_urls =
                get_service_endpoint_urls(&service, &deployment_name, client.clone()).await?;
            endpoints.extend(service_endpoint_urls);
        }

        let mut extra_infos = vec!["Third party service".to_string()];
        let containers = minio_deployment
            .spec
            .unwrap()
            .template
            .spec
            .unwrap()
            .containers;
        if let Some(minio_container) = containers.iter().find(|c| c.name == "minio") {
            if let Some(env) = &minio_container.env {
                let admin_user = env.iter().find(|e| e.name == "MINIO_ROOT_USER");
                let admin_password = env.iter().find(|e| e.name == "MINIO_ROOT_PASSWORD");

                if let (Some(admin_user), Some(admin_password)) = (admin_user, admin_password) {
                    let admin_user = admin_user
                        .value_from
                        .as_ref()
                        .ok_or("MinIO admin user env var needs to have an valueFrom entry")?
                        .secret_key_ref
                        .as_ref()
                        .ok_or("MinIO admin user env var needs to have an secretKeyRef in the valueFrom entry")?;
                    let admin_password = admin_password
                        .value_from
                        .as_ref()
                        .ok_or("MinIO admin password env var needs to have an valueFrom entry")?
                        .secret_key_ref
                        .as_ref()
                        .ok_or("MinIO admin password env var needs to have an secretKeyRef in the valueFrom entry")?;

                    let api: Api<Secret> = Api::namespaced(client.clone(), &deployment_namespace);
                    let admin_user_secret = api.get(admin_user.name.as_ref().unwrap()).await;
                    let admin_password_secret =
                        api.get(admin_password.name.as_ref().unwrap()).await;

                    if let (
                        Ok(Secret {
                            data: Some(admin_user_secret_data),
                            ..
                        }),
                        Ok(Secret {
                            data: Some(admin_password_secret_data),
                            ..
                        }),
                    ) = (admin_user_secret, admin_password_secret)
                    {
                        let admin_user = admin_user_secret_data
                            .get(&admin_user.key)
                            .map(|b| String::from_utf8(b.clone().0).unwrap())
                            .unwrap_or_default();
                        let admin_password = if redact_credentials {
                            REDACTED_PASSWORD.to_string()
                        } else {
                            admin_password_secret_data
                                .get(&admin_password.key)
                                .map(|b| String::from_utf8(b.clone().0).unwrap())
                                .unwrap_or_default()
                        };
                        extra_infos.push(format!(
                            "Admin user: {admin_user}, password: {admin_password}"
                        ));
                    }
                }
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
