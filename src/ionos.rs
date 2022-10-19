use std::error::Error;

use k8s_openapi::api::apps::v1::Deployment;
use kube::{api::ListParams, Api, ResourceExt};
use lazy_static::lazy_static;
use regex::Regex;

/// We asked the IONOS guys and agreed an this way of identifying a managed stackable cluster
/// One idea was to give the Namespace `stackable-operators` as special label
///
/// The current solution lists the deployment in the `stackable-operators` namespace and searches for matching names
/// This not the ideal solution and should be improved when there are better points of identifying a managed stackable cluster
pub async fn detect_ionos_managed_stackable_operators() -> Result<Vec<String>, Box<dyn Error>> {
    lazy_static! {
        static ref OPERATOR_DEPLOYMENT_REGEX: Regex =
            Regex::new("dp-[0-9a-f]{20}-([a-z-]+)-operator-deployment").unwrap();
    }

    let mut operators = Vec::new();

    let client = crate::kube::get_client().await?;
    let deployments = Api::<Deployment>::namespaced(client, "stackable-operators")
        .list(&ListParams::default())
        .await?;
    for deployment in deployments {
        let deployment_name = deployment.name_unchecked();
        if let Some(operator_name) = OPERATOR_DEPLOYMENT_REGEX
            .captures(&deployment_name)
            .and_then(|cap| cap.get(1).map(|m| m.as_str()))
        {
            operators.push(operator_name.to_string());
        }
    }

    Ok(operators)
}
