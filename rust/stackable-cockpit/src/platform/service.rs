//! Please be aware that this file is mostly copy/paste from <https://github.com/stackabletech/stackablectl/blob/eda45945cfcf5c6581cf1b88c782d98fada8065f/src/kube.rs#L48-L187>
//! It does only support services of type NodePort and LoadBalancer.
//! This code will be removed entirely once listener-op is implemented in all operators,
//! the endpoints will than be read by some sort of listener-operator API.
//!
//! So there is no point in optimizing this code or to blame the ones who have wrote it ;P

use std::collections::HashMap;

use indexmap::IndexMap;
use snafu::{OptionExt, ResultExt, Snafu};
use stackable_operator::{
    k8s_openapi::api::core::v1::{Service, ServiceSpec},
    kube::{self, ResourceExt, api::ListParams},
};
use tracing::{debug, warn};

use crate::utils::k8s::{self, Client, ListParamsExt};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("failed to fetch data from Kubernetes API"))]
    KubeClientFetch {
        #[snafu(source(from(k8s::Error, Box::new)))]
        source: Box<k8s::Error>,
    },

    #[snafu(display("missing namespace for service {service:?}"))]
    MissingServiceNamespace { service: String },

    #[snafu(display("missing spec for service {service:?}"))]
    MissingServiceSpec { service: String },

    #[snafu(display("failed to get status of node {node_name:?}"))]
    GetNodeStatus { node_name: String },

    #[snafu(display("failed to get address of node {node_name:?}"))]
    GetNodeAddress { node_name: String },

    #[snafu(display("failed to find an ExternalIP or InternalIP for node {node_name:?}"))]
    NoIpForNode { node_name: String },

    #[snafu(display("failed to find node {node_name:?} in node_name_ip_mapping"))]
    NodeMissingInIpMapping { node_name: String },
}

pub async fn get_endpoints(
    client: &Client,
    product_name: &str,
    object_name: &str,
    object_namespace: &str,
) -> Result<IndexMap<String, String>, Error> {
    let list_params =
        ListParams::from_product(product_name, Some(object_name), k8s::ProductLabel::Name);

    let listeners = client
        .list_listeners(Some(object_namespace), &list_params)
        .await;
    let listeners = match listeners {
        Ok(ok) => Ok(ok.items),
        Err(k8s::Error::KubeClientFetch { source })
            if matches!(&*source, kube::Error::Api(err) if err.code == 404) =>
        {
            // In case the listener-operator is not installed, this will return a 404. We should not fail, as this causes
            // stackablectl to fail with ApiError 404 on clusters without listener-operator.
            Ok(Vec::new())
        }
        Err(err) => Err(err),
    }
    .context(KubeClientFetchSnafu)?;

    let mut endpoints = IndexMap::new();
    for listener in &listeners {
        let Some(display_name) = display_name_for_listener_name(&listener.name_any(), object_name)
        else {
            continue;
        };
        let Some(listener_status) = &listener.status else {
            continue;
        };

        for address in listener_status.ingress_addresses.iter().flatten() {
            for port in &address.ports {
                let text = format!("{display_name}-{port_name}", port_name = port.0);
                let endpoint_url = endpoint_url(&address.address, *port.1, port.0);
                endpoints.insert(text, endpoint_url);
            }
        }
    }

    // Ideally we use listener-operator everywhere, afterwards we can remove the whole k8s Services handling below.
    // Currently the Services created by listener-op are missing the recommended labels, so this early exit in case we
    // find Listeners is currently not required. However, once we add the recommended labels to the k8s Services, we
    // would have duplicated entries (one from the Listener and one from the Service). Because of this we don't look at
    // the Services in case we found Listeners!
    if !listeners.is_empty() {
        return Ok(endpoints);
    }

    let services = client
        .list_services(Some(object_namespace), &list_params)
        .await
        .context(KubeClientFetchSnafu)?;

    for service in services {
        match get_endpoint_urls(client, &service, object_name).await {
            Ok(urls) => endpoints.extend(urls),
            Err(err) => warn!(
                "Failed to get endpoint_urls of service {service_name}: {err}",
                service_name = service.name_unchecked(),
            ),
        }
    }

    Ok(endpoints)
}

pub async fn get_endpoint_urls(
    client: &Client,
    service: &Service,
    referenced_object_name: &str,
) -> Result<IndexMap<String, String>, Error> {
    let service_name = service.name_unchecked();

    let service_namespace = service.namespace().context(MissingServiceNamespaceSnafu {
        service: service_name.clone(),
    })?;

    let service_spec = service.spec.as_ref().context(MissingServiceSpecSnafu {
        service: service_name.clone(),
    })?;

    let endpoints = match service_spec.type_.as_deref() {
        Some("NodePort") => {
            get_endpoint_urls_for_nodeport(
                client,
                &service_name,
                service_spec,
                &service_namespace,
                referenced_object_name,
            )
            .await?
        }
        Some("LoadBalancer") => {
            get_endpoint_urls_for_loadbalancer(
                &service_name,
                service,
                service_spec,
                referenced_object_name,
            )
            .await?
        }
        _ => IndexMap::new(),
    };

    Ok(endpoints)
}

pub async fn get_endpoint_urls_for_nodeport(
    client: &Client,
    service_name: &str,
    service_spec: &ServiceSpec,
    service_namespace: &str,
    referenced_object_name: &str,
) -> Result<IndexMap<String, String>, Error> {
    let endpoints = client
        .get_endpoints(service_namespace, service_name)
        .await
        .context(KubeClientFetchSnafu)?;

    let node_name = match &endpoints.subsets {
        Some(subsets) if subsets.len() == 1 => match &subsets[0].addresses {
            Some(addresses) if !addresses.is_empty() => match &addresses[0].node_name {
                Some(node_name) => node_name,
                None => {
                    warn!(
                        "Could not determine the node the endpoint {service_name} is running on because the address of the subset didn't had a node name"
                    );
                    return Ok(IndexMap::new());
                }
            },
            Some(_) => {
                warn!(
                    "Could not determine the node the endpoint {service_name} is running on because the subset had no addresses"
                );
                return Ok(IndexMap::new());
            }
            None => {
                warn!(
                    "Could not determine the node the endpoint {service_name} is running on because subset had no addresses. Is the service {service_name} up and running?"
                );
                return Ok(IndexMap::new());
            }
        },
        Some(subsets) => {
            warn!(
                "Could not determine the node the endpoint {service_name} is running on because endpoints consists of {num_subsets} subsets",
                num_subsets = subsets.len()
            );
            return Ok(IndexMap::new());
        }
        None => {
            warn!(
                "Could not determine the node the endpoint {service_name} is running on because the endpoint has no subset. Is the service {service_name} up and running?"
            );
            return Ok(IndexMap::new());
        }
    };

    let node_ip = get_node_ip(client, node_name).await?;

    let mut endpoints = IndexMap::new();
    for service_port in service_spec.ports.iter().flatten() {
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

                let endpoint = endpoint_url(&node_ip, node_port, &port_name);
                endpoints.insert(endpoint_name, endpoint);
            }
            None => debug!("Could not get endpoint_url as service {service_name} has no nodePort"),
        }
    }

    Ok(endpoints)
}

pub async fn get_endpoint_urls_for_loadbalancer(
    service_name: &str,
    service: &Service,
    service_spec: &ServiceSpec,
    referenced_object_name: &str,
) -> Result<IndexMap<String, String>, Error> {
    let mut endpoints = IndexMap::new();

    let lb_host = service
        .status
        .as_ref()
        .and_then(|s| s.load_balancer.as_ref())
        .and_then(|l| l.ingress.as_ref())
        .and_then(|l| l.first());

    if let Some(lb_host) = lb_host {
        let lb_host = lb_host.hostname.as_ref().or(lb_host.ip.as_ref());
        if let Some(lb_host) = lb_host {
            for service_port in service_spec.ports.iter().flatten() {
                let lb_port = service_port.port;

                let endpoint_name = service_name
                    .trim_start_matches(referenced_object_name)
                    .trim_start_matches('-');

                let port_name = service_port
                    .name
                    .clone()
                    .unwrap_or_else(|| lb_port.to_string());
                let endpoint_name = if endpoint_name.is_empty() {
                    port_name.clone()
                } else {
                    format!("{endpoint_name}-{port_name}")
                };

                let endpoint = endpoint_url(lb_host, lb_port, &port_name);
                endpoints.insert(endpoint_name, endpoint);
            }
        }
    }

    Ok(endpoints)
}

async fn get_node_ip(client: &Client, node_name: &str) -> Result<String, Error> {
    let node_name_ip_mapping = get_node_name_ip_mapping(client).await?;

    match node_name_ip_mapping.get(node_name) {
        Some(node_ip) => Ok(node_ip.to_string()),
        None => NodeMissingInIpMappingSnafu { node_name }.fail(),
    }
}

// TODO(sbernauer): Add caching. Not going to do so now, as listener-op
// will replace this code entirely anyway.
async fn get_node_name_ip_mapping(client: &Client) -> Result<HashMap<String, String>, Error> {
    let nodes = client.list_nodes().await.context(KubeClientFetchSnafu)?;

    let mut result = HashMap::new();
    for node in nodes {
        let node_name = node.name_unchecked();
        let preferred_node_ip = node
            .status
            .context(GetNodeStatusSnafu {
                node_name: node_name.to_string(),
            })?
            .addresses
            .context(GetNodeAddressSnafu {
                node_name: node_name.to_string(),
            })?
            .iter()
            .filter(|address| address.type_ == "InternalIP" || address.type_ == "ExternalIP")
            .min_by_key(|address| &address.type_) // ExternalIP (which we want) is lower than InternalIP
            .map(|address| address.address.clone())
            .context(NoIpForNodeSnafu {
                node_name: node_name.to_string(),
            })?;
        result.insert(node_name, preferred_node_ip);
    }

    Ok(result)
}

fn endpoint_url(endpoint_host: &str, endpoint_port: i32, port_name: &str) -> String {
    // TODO: Consolidate web-ui port names in operators based on decision in arch meeting from 2022-08-10
    // For Superset: https://github.com/stackabletech/superset-operator/issues/248
    // For Airflow: https://github.com/stackabletech/airflow-operator/issues/146
    // As we still support older operator versions we need to also include the "old" way of naming
    if port_name == "http"
        || port_name.starts_with("http-")
        || port_name == "ui-http"
        || port_name == "ui"
        || port_name == "airflow"
        || port_name == "superset"
    {
        format!("http://{endpoint_host}:{endpoint_port}")
    } else if port_name == "https" || port_name.starts_with("https-") {
        format!("https://{endpoint_host}:{endpoint_port}")
    } else {
        format!("{endpoint_host}:{endpoint_port}")
    }
}

/// Listener names usually have the pattern `listener-simple-hdfs-namenode-default-0` or
/// `simple-hdfs-datanode-default-0-listener`, so we can strip everything before the first occurrence of
/// the stacklet name (`simple-hdfs` in this case). After that it actually get's pretty hard.
/// This truncation is *not* ideal, however we only have implemented listener-operator for HDFS so far,
/// so better to have support for that than nothing :)
fn display_name_for_listener_name(listener_name: &str, object_name: &str) -> Option<String> {
    let (_, display_name) = listener_name.split_once(object_name)?;
    Some(display_name.trim_start_matches('-').to_owned())
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;

    #[rstest]
    // These are all the listener names implemented so far (only HDFS is using listener-operator). In the future more
    // test-case should be added.
    #[case("listener-simple-hdfs-namenode-default-0", "simple-hdfs", Some("namenode-default-0".to_string()))]
    #[case("listener-simple-hdfs-namenode-default-1", "simple-hdfs", Some("namenode-default-1".to_string()))]
    // FIXME: Come up with a more clever strategy to remove the `-listener` suffix. I would prefer to wait until we
    // actually have more products using listener-op to not accidentally strip to much.
    #[case("simple-hdfs-datanode-default-0-listener", "simple-hdfs", Some("datanode-default-0-listener".to_string()))]
    fn test_display_name_for_listener_name(
        #[case] listener_name: &str,
        #[case] object_name: &str,
        #[case] expected: Option<String>,
    ) {
        let output = display_name_for_listener_name(listener_name, object_name);
        assert_eq!(output, expected);
    }
}
