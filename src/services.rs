use std::error::Error;

use clap::Parser;
use indexmap::IndexMap;
use k8s_openapi::api::{apps::v1::Deployment, core::v1::Secret};
use kube::{
    api::{DynamicObject, GroupVersionKind, ListParams},
    core::ErrorResponse,
    Api, ResourceExt,
};
use lazy_static::lazy_static;
use log::{debug, warn};
use serde::Serialize;

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

        /// Don't show the product versions in the output
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
                list_services(*all_namespaces, *redact_credentials, *show_versions, output).await?
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

                        if endpoint.is_none() && extra_info.is_none() {
                            break;
                        }

                        println!(
                            "                                                                                     {:50} {}",
                            endpoint
                                .map(|(name, url)| { format!("{:20} {url}", format!("{name}:")) })
                                .unwrap_or_default(),
                            extra_info.map(|s| s.to_string()).unwrap_or_default(),
                        );
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

pub async fn get_stackable_services(
    namespaced: bool,
    redact_credentials: bool,
    show_versions: bool,
) -> Result<IndexMap<String, Vec<InstalledProduct>>, Box<dyn Error>> {
    let mut result = IndexMap::new();
    let namespace = NAMESPACE.lock().unwrap().clone();

    let client = get_client().await?;

    for (product_name, product_gvk) in STACKABLE_PRODUCT_CRDS.iter() {
        let api_resource = kube::core::discovery::ApiResource::from_gvk(product_gvk);
        let api: Api<DynamicObject> = match namespaced {
            true => Api::namespaced_with(client.clone(), &namespace, &api_resource),
            false => Api::all_with(client.clone(), &api_resource),
        };
        let objects = api.list(&ListParams::default()).await;
        match objects {
            Ok(objects) => {
                let mut installed_products = Vec::new();
                for object in objects {
                    let object_name = object.name();
                    let object_namespace = object.namespace();

                    let service_names = get_service_names(&object_name, product_name);
                    let extra_infos =
                        get_extra_infos(product_name, &object, redact_credentials, show_versions)
                            .await?;

                    let mut endpoints = IndexMap::new();
                    for service_name in service_names {
                        let service_endpoint_urls =
                            get_service_endpoint_urls(&service_name, &object_name, object_namespace
                                .as_ref()
                                .expect("Failed to get the namespace of object {object_name} besides it having an service")
                            , client.clone())
                                .await;
                        match service_endpoint_urls {
                            Ok(service_endpoint_urls) => endpoints.extend(service_endpoint_urls),
                            Err(err) => warn!(
                                "Failed to get endpoint_urls of service {service_name}: {err}"
                            ),
                        }
                    }
                    let product = InstalledProduct {
                        name: object_name,
                        namespace: object_namespace,
                        endpoints,
                        extra_infos,
                    };
                    installed_products.push(product);
                }
                result.insert(product_name.to_string(), installed_products);
            }
            Err(kube::Error::Api(ErrorResponse { code: 404, .. })) => {
                debug!("ProductCRD for product {product_name} not installed");
            }
            Err(err) => {
                return Err(Box::new(err));
            }
        }
    }

    Ok(result)
}

pub fn get_service_names(product_name: &str, product: &str) -> Vec<String> {
    match product {
        "airflow" => vec![format!("{product_name}-webserver")],
        "druid" => vec![
            format!("{product_name}-router"),
            format!("{product_name}-coordinator"),
        ],
        "hbase" => vec![product_name.to_string()],
        "hdfs" => vec![
            format!("{product_name}-datanode-default-0"),
            format!("{product_name}-namenode-default-0"),
            format!("{product_name}-journalnode-default-0"),
        ],
        "hive" => vec![product_name.to_string()],
        "nifi" => vec![product_name.to_string()],
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
    redact_credentials: bool,
    show_versions: bool,
) -> Result<Vec<String>, Box<dyn Error>> {
    let mut result = Vec::new();

    match product {
        "airflow" | "superset" => {
            if let Some(secret_name) = product_crd.data["spec"]["credentialsSecret"].as_str() {
                let credentials = get_credentials_from_secret(
                    secret_name,
                    product_crd.namespace().unwrap().as_str(),
                    "adminUser.username",
                    "adminUser.password",
                    redact_credentials,
                )
                .await?;

                if let Some((username, password)) = credentials {
                    result.push(format!("admin user: {username}, password: {password}"));
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
    let secret_data = secret.data.unwrap();

    match (secret_data.get(username_key), secret_data.get(password_key)) {
        (Some(username), Some(password)) => {
            let username = String::from_utf8(username.0.clone()).unwrap();
            let password = if redact_credentials {
                REDACTED_PASSWORD.to_string()
            } else {
                String::from_utf8(password.0.clone()).unwrap()
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
                        .unwrap()
                        .secret_key_ref
                        .as_ref()
                        .unwrap();
                    let admin_password = admin_password
                        .value_from
                        .as_ref()
                        .unwrap()
                        .secret_key_ref
                        .as_ref()
                        .unwrap();

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
                            "admin user: {admin_user}, password: {admin_password}"
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
