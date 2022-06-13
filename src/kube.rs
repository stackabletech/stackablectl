use crate::{
    helpers,
    services::{get_service_names, InstalledProduct, PRODUCT_CRDS},
    NAMESPACE,
};
use cached::proc_macro::cached;
use core::panic;
use indexmap::IndexMap;
use k8s_openapi::api::core::v1::{Endpoints, Node};
use kube::{
    api::{DynamicObject, ListParams},
    core::ErrorResponse,
    Api, Client, ResourceExt,
};
use log::{debug, warn};
use std::{collections::HashMap, error::Error, vec};

/// This function currently uses `kubectl apply`.
/// In the future we want to switch to kube-rs or something else to not require the user to install kubectl.
pub fn deploy_manifest(yaml: &str) {
    let namespace = NAMESPACE.lock().unwrap();
    helpers::execute_command_with_stdin(
        vec!["kubectl", "apply", "-n", &namespace, "-f", "-"],
        yaml,
    );
}

pub async fn get_services(
    namespaced: bool,
) -> Result<IndexMap<String, Vec<InstalledProduct>>, Box<dyn Error>> {
    let mut result = IndexMap::new();
    let namespace = NAMESPACE.lock().unwrap().clone();

    let client = get_client().await?;

    for (product_name, product_gvk) in PRODUCT_CRDS.iter() {
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
                    let mut endpoints = IndexMap::new();
                    for service_name in service_names {
                        let service_endpoint_urls =
                            get_service_endpoint_urls(&service_name, object_namespace.as_ref().expect("Failed to get Namespace of object {object_name} besides it having an service"), client.clone())
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

pub async fn get_service_endpoint_urls(
    service_name: &str,
    namespace: &str,
    client: Client,
) -> Result<IndexMap<String, String>, Box<dyn Error>> {
    let service_api: Api<k8s_openapi::api::core::v1::Service> =
        Api::namespaced(client.clone(), namespace);
    let service = service_api.get(service_name).await?;

    let endpoints_api: Api<Endpoints> = Api::namespaced(client.clone(), namespace);
    let endpoints = endpoints_api.get(service_name).await?;

    let node_name = match &endpoints.subsets {
        Some(subsets) if subsets.len() == 1 => match &subsets[0].addresses {
            Some(addresses) if addresses.len() == 1 => match &addresses[0].node_name {
                Some(node_name) => node_name,
                None => {
                    warn!("Could not determine the node the endpoint is running on because the address of the subset didn't had a node name");
                    return Ok(IndexMap::new());
                }
            },
            Some(_) => {
                warn!("Could not determine the node the endpoint is running on because subset had multiple addresses");
                return Ok(IndexMap::new());
            }
            None => {
                warn!("Could not determine the node the endpoint is running on because subset had no addresses");
                return Ok(IndexMap::new());
            }
        },
        Some(_) => {
            warn!("Could not determine the node the endpoint is running on because endpoints consists of multiple subsets");
            return Ok(IndexMap::new());
        }
        None => {
            warn!("Could not determine the node the endpoint is running on because the endpoint has no subset");
            return Ok(IndexMap::new());
        }
    };

    let node_ip = get_node_ip(node_name).await;

    let mut result = IndexMap::new();
    for service_port in service.spec.unwrap().ports.unwrap_or_default() {
        let port_name = service_port.name.unwrap_or_else(|| "".to_string());
        let port_number = service_port.port;
        let node_port_number = service_port.node_port.unwrap_or(port_number); // TODO: Is this correct behavior?

        result.insert(port_name, format!("http://{node_ip}:{node_port_number} "));
    }

    Ok(result)
}

async fn get_node_ip(node_name: &str) -> String {
    let node_name_ip_mapping = get_node_name_ip_mapping().await;
    match node_name_ip_mapping.get(node_name) {
        Some(node_ip) => node_ip.to_string(),
        None => panic!("Failed to find node {node_name} in node_name_ip_mapping"),
    }
}

/// Not returning an Result<HashMap<String, String>, Error> because i couldn't get it to work with #[cached]
#[cached]
async fn get_node_name_ip_mapping() -> HashMap<String, String> {
    let client = get_client()
        .await
        .expect("Failed to create kubernetes client");
    let node_api: Api<Node> = Api::all(client);
    let nodes = node_api
        .list(&ListParams::default())
        .await
        .expect("Failed to list kubernetes nodes");

    let mut result = HashMap::new();
    for node in nodes {
        let node_name = node.name();
        let preferred_node_ip = node
            .status
            .unwrap()
            .addresses
            .unwrap_or_else(|| panic!("Failed to get address of node {node_name}"))
            .iter()
            .filter(|address| address.type_ == "InternalIP" || address.type_ == "ExternalIP")
            .max_by_key(|address| &address.type_)
            .map(|address| address.address.clone()) // InternalIP is lower than ExternalIP
            .unwrap_or_else(|| {
                panic!("Could not find a InternalIP or ExternalIP for node {node_name}")
            });
        result.insert(node_name, preferred_node_ip);
    }

    result
}

async fn get_client() -> Result<Client, Box<dyn Error>> {
    Ok(Client::try_default().await?)
}
