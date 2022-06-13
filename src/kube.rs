use indexmap::IndexMap;
use kube::{
    api::{DynamicObject, GroupVersionKind, ListParams},
    core::ErrorResponse,
    Api, Client, ResourceExt,
};
use lazy_static::{__Deref, lazy_static};
use log::debug;
use serde::{Deserialize, Serialize};
use std::{error::Error, vec};

use crate::{helpers, services::InstalledProduct, NAMESPACE};

lazy_static! {
    pub static ref PRODUCT_CRDS: IndexMap<&'static str, GroupVersionKind> = IndexMap::from([
        (
            "hive",
            GroupVersionKind {
                group: "hive.stackable.tech".to_string(),
                version: "v1alpha1".to_string(),
                kind: "HiveCluster".to_string(),
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
            "doesnotexist",
            GroupVersionKind {
                group: "doesnotexist.stackable.tech".to_string(),
                version: "v1alpha1".to_string(),
                kind: "DoesnotexistCluster".to_string(),
            }
        ),
    ]);
}

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

    let client = get_client().await?;

    for (product_name, product_gvk) in PRODUCT_CRDS.iter() {
        let api_resource = kube::core::discovery::ApiResource::from_gvk(product_gvk);
        let api: Api<DynamicObject> = match namespaced {
            true => Api::namespaced_with(
                client.clone(),
                NAMESPACE.lock().unwrap().deref(),
                &api_resource,
            ),
            false => Api::all_with(client.clone(), &api_resource),
        };
        let objects = api.list(&ListParams::default()).await;
        match objects {
            Ok(objects) => {
                let installed_products = objects
                    .iter()
                    .map(|o| {
                        let endpoints = IndexMap::from([
                            ("web-ui".to_string(), "http://todo.com".to_string()),
                            ("rpc".to_string(), "http://todo.com".to_string()),
                        ]);
                        InstalledProduct {
                            name: o.name(),
                            namespace: o.namespace(),
                            endpoints,
                        }
                    })
                    .collect::<Vec<_>>();
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
