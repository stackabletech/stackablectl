use clap::ValueEnum;
use snafu::ResultExt;
use stackable_operator::{
    k8s_openapi::api::core::v1::Node,
    kube::{Api, Client, api::ListParams},
};
use tokio::sync::OnceCell;
use tracing::{debug, info, instrument};
use yaml_serde::Value;

pub static LISTENER_CLASS_PRESET: OnceCell<ListenerClassPreset> = OnceCell::const_new();

/// Represents the `preset` value in the Listener Operator Helm Chart
#[derive(Copy, Clone, Debug, ValueEnum)]
pub enum ListenerClassPreset {
    None,
    StableNodes,
    EphemeralNodes,
}

impl ListenerClassPreset {
    pub fn as_helm_value(&self) -> Value {
        let preset_value = match self {
            Self::None => "none",
            Self::StableNodes => "stable-nodes",
            Self::EphemeralNodes => "ephemeral-nodes",
        };
        Value::String(preset_value.to_string())
    }
}

#[instrument]
pub async fn determine_and_store_listener_class_preset(from_cli: Option<&ListenerClassPreset>) {
    if let Some(from_cli) = from_cli {
        LISTENER_CLASS_PRESET
            .set(*from_cli)
            .expect("LISTENER_CLASS_PRESET should be unset");
        return;
    }

    let kubernetes_environment = guess_kubernetes_environment().await.unwrap_or_else(|err| {
        info!("failed to determine Kubernetes environment, using defaults: {err:#?}");
        KubernetesEnvironment::Unknown
    });
    let listener_class_preset = match kubernetes_environment {
        // Kind does not support LoadBalancers out of the box, so avoid that
        KubernetesEnvironment::Kind => ListenerClassPreset::StableNodes,
        // Minikube does support LoadBalancers via "minikube tunnel", but that requires additional setup
        // and we assume that users running Minikube with stackablectl want to keep things simple.
        KubernetesEnvironment::Minikube => ListenerClassPreset::StableNodes,
        // LoadBalancer support in k3s is optional, so let's be better safe than sorry and not use
        // them
        KubernetesEnvironment::K3s => ListenerClassPreset::StableNodes,
        // Weekly node rotations and LoadBalancer support
        KubernetesEnvironment::Ionos => ListenerClassPreset::EphemeralNodes,
        // Don't pin nodes and assume we have LoadBalancer support
        KubernetesEnvironment::Unknown => ListenerClassPreset::EphemeralNodes,
    };
    debug!(
        preset = ?listener_class_preset,
        kubernetes.environment = ?kubernetes_environment,
        "Using ListenerClass preset"
    );

    LISTENER_CLASS_PRESET
        .set(listener_class_preset)
        .expect("LISTENER_CLASS_PRESET should be unset");
}

#[derive(Debug)]
enum KubernetesEnvironment {
    Kind,
    K3s,
    Ionos,
    Minikube,
    Unknown,
}

/// Tries to guess what Kubernetes environment stackablectl is connecting to.
///
/// Returns an error in case anything goes wrong. This could e.g. be the case in case no
/// Kubernetes context is configured, stackablectl is missing RBAC permission to retrieve nodes or
/// simply a network error.
#[instrument]
async fn guess_kubernetes_environment() -> Result<KubernetesEnvironment, snafu::Whatever> {
    let client = Client::try_default()
        .await
        .whatever_context("failed to construct Kubernetes client")?;
    let node_api: Api<Node> = Api::all(client);
    let nodes = node_api
        .list(&ListParams::default())
        .await
        .whatever_context("failed to list Kubernetes nodes")?;

    for node in nodes {
        if let Some(spec) = node.spec {
            if let Some(provider_id) = spec.provider_id {
                if provider_id.starts_with("kind://") {
                    return Ok(KubernetesEnvironment::Kind);
                } else if provider_id.starts_with("k3s://") {
                    return Ok(KubernetesEnvironment::K3s);
                } else if provider_id.starts_with("ionos://") {
                    return Ok(KubernetesEnvironment::Ionos);
                }
            }
        }

        // Check labels for minikube (since minikube doesn't set providerID)
        if let Some(labels) = node.metadata.labels.as_ref() {
            if labels.contains_key("minikube.k8s.io/version") {
                return Ok(KubernetesEnvironment::Minikube);
            }
        }
    }

    Ok(KubernetesEnvironment::Unknown)
}
