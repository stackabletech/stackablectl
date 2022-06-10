use indexmap::IndexMap;
use kube::{api::ListParams, Api, Client, ResourceExt};
use serde::{Deserialize, Serialize};
use std::{error::Error, vec};

use crate::{helpers, NAMESPACE};

/// This function currently uses `kubectl apply`.
/// In the future we want to switch to kube-rs or something else to not require the user to install kubectl.
pub fn deploy_manifest(yaml: &str) {
    let namespace = NAMESPACE.lock().unwrap();
    helpers::execute_command_with_stdin(
        vec!["kubectl", "apply", "-n", &namespace, "-f", "-"],
        yaml,
    );
}

pub async fn get_services(namespaced: bool) -> Result<IndexMap<String, Service>, Box<dyn Error>> {
    let client = get_client().await?;

    let services: Api<k8s_openapi::api::core::v1::Service> = match namespaced {
        true => Api::default_namespaced(client),
        false => Api::all(client),
    };

    Ok(services
        .list(&ListParams::default())
        .await?
        .iter()
        .map(|service| {
            let ports = service
                .spec
                .as_ref()
                .unwrap()
                .ports
                .as_ref()
                .unwrap()
                .iter()
                .map(|port| ServicePort {
                    name: port.name.clone(),
                })
                .collect::<Vec<_>>();
            (
                service.name(),
                Service {
                    namespace: service.namespace().unwrap(),
                    ports,
                },
            )
        })
        .collect())
}

async fn get_client() -> Result<Client, Box<dyn Error>> {
    Ok(Client::try_default().await?)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Service {
    pub namespace: String,
    pub ports: Vec<ServicePort>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ServicePort {
    pub name: Option<String>,
}
