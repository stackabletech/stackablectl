use std::error::Error;

use k8s_openapi::api::core::v1::{Secret, Service};
use kube::{api::ListParams, Api, ResourceExt};

use crate::{
    kube::{get_client, get_service_endpoint_urls},
    NAMESPACE,
};

use super::{InstalledProduct, REDACTED_PASSWORD};

pub async fn get_opensearch_dashboards_services(
    namespaced: bool,
    redact_credentials: bool,
) -> Result<Vec<InstalledProduct>, Box<dyn Error>> {
    let client = get_client().await?;
    let list_params = ListParams::default().labels("app.kubernetes.io/name=opensearch-dashboards");

    let mut result = Vec::new();

    let service_api: Api<Service> = if namespaced {
        Api::namespaced(client.to_owned(), NAMESPACE.lock()?.as_str())
    } else {
        Api::all(client.clone())
    };

    let services = service_api.list(&list_params).await?;
    for service in services {
        let namespace = service.namespace().unwrap();
        let endpoints =
            get_service_endpoint_urls(&service, &service.name_unchecked(), client.to_owned())
                .await?;

        let annotations = service.annotations();

        let mut extra_infos = vec!["Third party service".to_string()];

        if let Some(http_endpoint) = endpoints.get("http") {
            if let Some(logs_endpoint) = annotations.get("stackable.tech/logging-view-logs") {
                extra_infos.push(format!("Logs view: {http_endpoint}{logs_endpoint}"));
            }
        }

        if let Some(credentials_secret) =
            annotations.get("stackable.tech/logging-credentials-secret")
        {
            let secret_api: Api<Secret> = Api::namespaced(client.clone(), &namespace);
            if let Ok(credentials) = secret_api.get(credentials_secret).await {
                if let Some(username) = credentials
                    .data
                    .as_ref()
                    .and_then(|data| data.get("username"))
                    .and_then(|value| String::from_utf8(value.0.to_owned()).ok())
                {
                    let mut credentials_info = format!("Username: {username}");
                    if redact_credentials {
                        credentials_info.push_str(&format!(", password: {REDACTED_PASSWORD}"));
                    } else if let Some(password) = credentials
                        .data
                        .as_ref()
                        .and_then(|data| data.get("password"))
                        .and_then(|value| String::from_utf8(value.0.to_owned()).ok())
                    {
                        credentials_info.push_str(&format!(", password: {password}"));
                    }
                    extra_infos.push(credentials_info);
                }
            }
        }

        let installed_product = InstalledProduct {
            name: service.name_unchecked(),
            namespace: service.namespace(),
            endpoints,
            extra_infos,
        };
        result.push(installed_product);
    }

    Ok(result)
}
