use indexmap::IndexMap;
use k8s_openapi::api::core::v1::Service;
use kube::{api::ListParams, Api, ResourceExt};
use std::{error::Error, vec};

use crate::{
    kube::{get_client, get_service_endpoint_urls},
    NAMESPACE,
};

use super::InstalledProduct;

const TRIM_SERVICE_NAME: &str = "-kube-prometheus-prometheus";

pub async fn get_prometheus_services(
    namespaced: bool,
    _redact_credentials: bool,
) -> Result<Vec<InstalledProduct>, Box<dyn Error>> {
    let client = get_client().await?;
    let list_params = ListParams::default()
        .labels("app.kubernetes.io/instance=prometheus,app=kube-prometheus-stack-prometheus");

    let mut result = Vec::new();

    let service_api: Api<Service> = match namespaced {
        true => Api::namespaced(client.clone(), NAMESPACE.lock()?.as_str()),
        false => Api::all(client.clone()),
    };
    let services = service_api.list(&list_params).await?;

    for service in services {
        let prometheus_name = service
            .name_unchecked()
            .trim_end_matches(TRIM_SERVICE_NAME)
            .to_string();

        let mut endpoints = IndexMap::new();
        let service_endpoint_urls = get_service_endpoint_urls(
            &service,
            &service.name_unchecked(), // We pass the full service name (instead of prometheus_name) in here to not get long endpoint names
            client.clone(),
        )
        .await?;
        endpoints.extend(service_endpoint_urls);

        let extra_infos = vec!["Third party service".to_string()];

        result.push(InstalledProduct {
            name: prometheus_name,
            namespace: service.namespace(),
            endpoints,
            extra_infos,
        });
    }
    Ok(result)
}
