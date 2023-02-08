use indexmap::IndexMap;
use k8s_openapi::api::core::v1::Service;
use kube::{api::ListParams, Api, ResourceExt};
use std::{error::Error, vec};

use crate::{
    kube::{get_client, get_service_endpoint_urls},
    NAMESPACE,
};

use super::{get_credentials_from_secret, InstalledProduct};

pub async fn get_grafana_services(
    namespaced: bool,
    redact_credentials: bool,
) -> Result<Vec<InstalledProduct>, Box<dyn Error>> {
    let client = get_client().await?;
    let list_params = ListParams::default().labels("app.kubernetes.io/name=grafana");

    let mut result = Vec::new();

    let service_api: Api<Service> = match namespaced {
        true => Api::namespaced(client.clone(), NAMESPACE.lock()?.as_str()),
        false => Api::all(client.clone()),
    };
    let services = service_api.list(&list_params).await?;

    for service in services {
        let mut endpoints = IndexMap::new();
        let service_endpoint_urls =
            get_service_endpoint_urls(&service, &service.name_unchecked(), client.clone()).await?;
        endpoints.extend(service_endpoint_urls);

        let mut extra_infos = vec!["Third party service".to_string()];

        // We assume the prom-operator was used for installation, in which case we know the secret name
        let credentials = get_credentials_from_secret(
            &service.name_unchecked(),
            service
                .namespace()
                .as_ref()
                .ok_or("Grafana service had no namespace set")?,
            "admin-user",
            "admin-password",
            redact_credentials,
        )
        .await?;

        if let Some((username, password)) = credentials {
            extra_infos.push(format!("Admin user: {username}, password: {password}"));
        }

        result.push(InstalledProduct {
            name: service.name_unchecked(),
            namespace: service.namespace(),
            endpoints,
            extra_infos,
        });
    }
    Ok(result)
}
