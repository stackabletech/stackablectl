use crate::{helpers, NAMESPACE};
use cached::proc_macro::cached;
use core::panic;
use indexmap::IndexMap;
use k8s_openapi::api::core::v1::{Endpoints, Node};
use kube::{api::ListParams, Api, Client, ResourceExt};
use log::warn;
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

pub async fn get_service_endpoint_urls(
    service_name: &str,
    object_name: &str,
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
            Some(addresses) => match &addresses[0].node_name {
                Some(node_name) => node_name,
                None => {
                    warn!("Could not determine the node the endpoint {service_name} is running on because the address of the subset didn't had a node name");
                    return Ok(IndexMap::new());
                }
            },
            None => {
                warn!("Could not determine the node the endpoint {service_name} is running on because subset had no addresses");
                return Ok(IndexMap::new());
            }
        },
        Some(_) => {
            warn!("Could not determine the node the endpoint {service_name} is running on because endpoints consists of multiple subsets");
            return Ok(IndexMap::new());
        }
        None => {
            warn!("Could not determine the node the endpoint {service_name} is running on because the endpoint has no subset");
            return Ok(IndexMap::new());
        }
    };

    let node_ip = get_node_ip(node_name).await;

    let mut result = IndexMap::new();
    for service_port in service.spec.unwrap().ports.unwrap_or_default() {
        match service_port.node_port {
            Some(node_port) => {
                let endpoint_name = service_name
                    .trim_start_matches(object_name)
                    .trim_start_matches('-');

                let port_name = service_port.name.unwrap_or_else(|| node_port.to_string());
                let endpoint_name = if endpoint_name.is_empty() {
                    port_name.clone()
                } else {
                    format!("{endpoint_name}-{port_name}")
                };
                let endpoint = match port_name.as_str() {
                    "http" => format!("http://{node_ip}:{node_port}"),
                    "https" => format!("https://{node_ip}:{node_port}"),
                    _ => format!("{node_ip}:{node_port}"),
                };

                result.insert(endpoint_name, endpoint);
            }
            None => warn!("Could not get endpoint_url as service {service_name} has no nodePort"),
        }
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
            .min_by_key(|address| &address.type_) // ExternalIP (which we want) is lower than InternalIP
            .map(|address| address.address.clone())
            .unwrap_or_else(|| {
                panic!("Could not find a InternalIP or ExternalIP for node {node_name}")
            });
        result.insert(node_name, preferred_node_ip);
    }

    result
}

pub async fn get_client() -> Result<Client, Box<dyn Error>> {
    Ok(Client::try_default().await?)
}
