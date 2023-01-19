use std::error::Error;

use indexmap::IndexMap;
use k8s_openapi::api::{
    apps::v1::{Deployment, StatefulSet},
    core::v1::{Container, Secret, Service},
};
use kube::{api::ListParams, Api, Client, ResourceExt};

use crate::{
    kube::{get_client, get_service_endpoint_urls},
    NAMESPACE,
};

use super::{InstalledProduct, REDACTED_PASSWORD};

pub async fn get_minio_services(
    namespaced: bool,
    redact_credentials: bool,
) -> Result<Vec<InstalledProduct>, Box<dyn Error>> {
    let client = get_client().await?;
    let list_params = ListParams::default().labels("app=minio");

    let mut result = Vec::new();

    // MinIO can either be installed in standalone mode which creates a Deployment
    // The other option is to run it in a distributed mode, which created a StatefulSet
    // So we have to check for both
    let deployment_api: Api<Deployment> = match namespaced {
        true => Api::namespaced(client.clone(), NAMESPACE.lock()?.as_str()),
        false => Api::all(client.clone()),
    };
    let deployments = deployment_api.list(&list_params).await?;
    for deployment in deployments {
        let installed_product = get_minio_service(
            &deployment.name_unchecked(),
            &deployment
                .namespace()
                .ok_or("MinIO deployment has no namespace")?,
            &deployment
                .spec
                .ok_or("MinIO deployment has no spec")?
                .template
                .spec
                .ok_or("MinIO deployment has no template spec")?
                .containers,
            client.clone(),
            redact_credentials,
        )
        .await?;
        result.push(installed_product);
    }

    let statefulset_api: Api<StatefulSet> = match namespaced {
        true => Api::namespaced(client.clone(), NAMESPACE.lock()?.as_str()),
        false => Api::all(client.clone()),
    };
    let statefulsets = statefulset_api.list(&list_params).await?;
    for statefulset in statefulsets {
        let installed_product = get_minio_service(
            &statefulset.name_unchecked(),
            &statefulset
                .namespace()
                .ok_or("MinIO statefulset has no namespace")?,
            &statefulset
                .spec
                .ok_or("MinIO statefulset has no spec")?
                .template
                .spec
                .ok_or("MinIO statefulset has no template spec")?
                .containers,
            client.clone(),
            redact_credentials,
        )
        .await?;
        result.push(installed_product);
    }

    Ok(result)
}

pub async fn get_minio_service(
    name: &str,
    namespace: &str,
    containers: &[Container],
    client: Client,
    redact_credentials: bool,
) -> Result<InstalledProduct, Box<dyn Error>> {
    let service_api: Api<Service> = Api::namespaced(client.clone(), namespace);
    let service_names = [name.to_string(), format!("{name}-console")];

    let mut endpoints = IndexMap::new();
    for service_name in service_names {
        let service = service_api.get(&service_name).await?;
        let service_endpoint_urls =
            get_service_endpoint_urls(&service, name, client.clone()).await?;
        endpoints.extend(service_endpoint_urls);
    }

    let mut extra_infos = vec!["Third party service".to_string()];
    if let Some(minio_container) = containers.iter().find(|c| c.name == "minio") {
        if let Some(env) = &minio_container.env {
            let admin_user = env.iter().find(|e| e.name == "MINIO_ROOT_USER");
            let admin_password = env.iter().find(|e| e.name == "MINIO_ROOT_PASSWORD");

            if let (Some(admin_user), Some(admin_password)) = (admin_user, admin_password) {
                let admin_user = admin_user
                    .value_from
                    .as_ref()
                    .ok_or("MinIO admin user env var needs to have an valueFrom entry")?
                    .secret_key_ref
                    .as_ref()
                    .ok_or("MinIO admin user env var needs to have an secretKeyRef in the valueFrom entry")?;
                let admin_password = admin_password
                    .value_from
                    .as_ref()
                    .ok_or("MinIO admin password env var needs to have an valueFrom entry")?
                    .secret_key_ref
                    .as_ref()
                    .ok_or("MinIO admin password env var needs to have an secretKeyRef in the valueFrom entry")?;

                let api: Api<Secret> = Api::namespaced(client.clone(), namespace);
                let admin_user_secret = api.get(admin_user.name.as_ref().unwrap()).await;
                let admin_password_secret = api.get(admin_password.name.as_ref().unwrap()).await;

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
                        "Admin user: {admin_user}, password: {admin_password}"
                    ));
                }
            }
        }
    }

    Ok(InstalledProduct {
        name: name.to_string(),
        namespace: Some(namespace.to_string()),
        endpoints,
        extra_infos,
    })
}
