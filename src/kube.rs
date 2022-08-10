use crate::NAMESPACE;
use cached::proc_macro::cached;
use indexmap::IndexMap;
use k8s_openapi::api::core::v1::{Endpoints, Node, Service};
use kube::{
    api::{DynamicObject, GroupVersionKind, ListParams, Patch, PatchParams, TypeMeta},
    discovery::Scope,
    Api, Client, Discovery, ResourceExt,
};
use log::{debug, warn};
use serde::Deserialize;
use std::{collections::HashMap, error::Error};

pub async fn deploy_manifests(yaml: &str) -> Result<(), Box<dyn Error>> {
    let namespace = NAMESPACE.lock()?.clone();
    let client = get_client().await?;
    let discovery = Discovery::new(client.clone()).run().await?;

    for manifest in serde_yaml::Deserializer::from_str(yaml) {
        let mut object = DynamicObject::deserialize(manifest)?;

        let gvk = gvk_of_typemeta(object.types.as_ref().ok_or(format!(
            "Failed to deploy manifest because type of object {object:?} is not set"
        ))?);
        let (resource, capabilities) = discovery.resolve_gvk(&gvk).ok_or(format!(
            "Failed to deploy manifest because the gvk {gvk:?} can not be resolved"
        ))?;

        let api: Api<DynamicObject> = match capabilities.scope {
            Scope::Cluster => {
                object.metadata.namespace = None;
                Api::all_with(client.clone(), &resource)
            }
            Scope::Namespaced => Api::namespaced_with(client.clone(), &namespace, &resource),
        };

        api.patch(
            &object.name(),
            &PatchParams::apply("stackablectl"),
            &Patch::Apply(object),
        )
        .await?;
    }

    Ok(())
}

pub async fn get_service_endpoint_urls(
    service: &Service,
    referenced_object_name: &str,
    client: Client,
) -> Result<IndexMap<String, String>, Box<dyn Error>> {
    let namespace = service
        .namespace()
        .ok_or(format!("Service {service:?} must have a namespace"))?;
    let service_name = service.name();

    let endpoints_api: Api<Endpoints> = Api::namespaced(client.clone(), &namespace);
    let endpoints = endpoints_api.get(&service_name).await?;

    let node_name = match &endpoints.subsets {
        Some(subsets) if subsets.len() == 1 => match &subsets[0].addresses {
            Some(addresses) if !addresses.is_empty() => match &addresses[0].node_name {
                Some(node_name) => node_name,
                None => {
                    warn!("Could not determine the node the endpoint {service_name} is running on because the address of the subset didn't had a node name");
                    return Ok(IndexMap::new());
                }
            },
            Some(_) => {
                warn!("Could not determine the node the endpoint {service_name} is running on because the subset had no addresses");
                return Ok(IndexMap::new());
            }
            None => {
                warn!("Could not determine the node the endpoint {service_name} is running on because subset had no addresses. Is the service {service_name} up and running?");
                return Ok(IndexMap::new());
            }
        },
        Some(subsets) => {
            warn!("Could not determine the node the endpoint {service_name} is running on because endpoints consists of {num_subsets} subsets", num_subsets=subsets.len());
            return Ok(IndexMap::new());
        }
        None => {
            warn!("Could not determine the node the endpoint {service_name} is running on because the endpoint has no subset. Is the service {service_name} up and running?");
            return Ok(IndexMap::new());
        }
    };

    let node_ip = get_node_ip(node_name).await?;

    let mut result = IndexMap::new();
    for service_port in service
        .spec
        .as_ref()
        .ok_or(format!("Service {service_name} had no spec"))?
        .ports
        .iter()
        .flatten()
    {
        match service_port.node_port {
            Some(node_port) => {
                let endpoint_name = service_name
                    .trim_start_matches(referenced_object_name)
                    .trim_start_matches('-');

                let port_name = service_port
                    .name
                    .clone()
                    .unwrap_or_else(|| service_port.port.to_string());
                let endpoint_name = if endpoint_name.is_empty() {
                    port_name.clone()
                } else {
                    format!("{endpoint_name}-{port_name}")
                };
                let endpoint = match port_name.as_str() {
                    // TODO: Consolidate web-ui port names in operators based on decision in arch meeting from 2022/08/10
                    // * In case a single http port call it http
                    // * If you have multiple http ports call it http-ui
                    // For Superset: https://github.com/stackabletech/superset-operator/issues/248
                    // For Airflow: https://github.com/stackabletech/airflow-operator/issues/146
                    // As we still support older operator versions we need to also include the "old" way of naming
                    "http" | "http-ui" | "ui" | "airflow" | "superset" => {
                        format!("http://{node_ip}:{node_port}")
                    }
                    "https" => format!("https://{node_ip}:{node_port}"),
                    _ => format!("{node_ip}:{node_port}"),
                };

                result.insert(endpoint_name, endpoint);
            }
            None => debug!("Could not get endpoint_url as service {service_name} has no nodePort"),
        }
    }

    Ok(result)
}

async fn get_node_ip(node_name: &str) -> Result<String, Box<dyn Error>> {
    let node_name_ip_mapping = get_node_name_ip_mapping().await?;

    match node_name_ip_mapping.get(node_name) {
        Some(node_ip) => Ok(node_ip.to_string()),
        None => Err(format!("Failed to find node {node_name} in node_name_ip_mapping").into()),
    }
}

#[cached]
async fn get_node_name_ip_mapping() -> Result<HashMap<String, String>, String> {
    let client = get_client()
        .await
        .map_err(|err| format!("Failed to create Kubernetes client: {err}"))?;
    let node_api: Api<Node> = Api::all(client);
    let nodes = node_api
        .list(&ListParams::default())
        .await
        .map_err(|err| format!("Failed to list Kubernetes nodes: {err}"))?;

    let mut result = HashMap::new();
    for node in nodes {
        let node_name = node.name();
        let preferred_node_ip = node
            .status
            .ok_or(format!("Failed to get status of node {node_name}"))?
            .addresses
            .ok_or(format!("Failed to get address of node {node_name}"))?
            .iter()
            .filter(|address| address.type_ == "InternalIP" || address.type_ == "ExternalIP")
            .min_by_key(|address| &address.type_) // ExternalIP (which we want) is lower than InternalIP
            .map(|address| address.address.clone())
            .ok_or(format!(
                "Could not find an ExternalIP or InternalIP for node {node_name}"
            ))?;
        result.insert(node_name, preferred_node_ip);
    }

    Ok(result)
}

pub async fn get_client() -> Result<Client, Box<dyn Error>> {
    Client::try_default()
        .await
        .map_err(|err| format! {"Failed to construct Kubernetes client: {err}"}.into())
}

fn gvk_of_typemeta(type_meta: &TypeMeta) -> GroupVersionKind {
    match type_meta.api_version.split_once('/') {
        Some((group, version)) => GroupVersionKind::gvk(group, version, &type_meta.kind),
        None => GroupVersionKind::gvk("", &type_meta.api_version, &type_meta.kind),
    }
}
