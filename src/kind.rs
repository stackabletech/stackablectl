use crate::helpers;
use log::{info, warn};

const DEFAULT_KIND_CLUSTER_NAME: &str = "stackable-data-platform";

const KIND_CLUSTER_DEFINITION: &str = r#"
kind: Cluster
apiVersion: kind.x-k8s.io/v1alpha4
nodes:
- role: control-plane
- role: worker
  kubeadmConfigPatches:
    - |
      kind: JoinConfiguration
      nodeRegistration:
        kubeletExtraArgs:
          node-labels: node=1,
- role: worker
  kubeadmConfigPatches:
    - |
      kind: JoinConfiguration
      nodeRegistration:
        kubeletExtraArgs:
          node-labels: node=2
- role: worker
  kubeadmConfigPatches:
    - |
      kind: JoinConfiguration
      nodeRegistration:
        kubeletExtraArgs:
          node-labels: node=3
"#;

pub fn handle_cli_arguments(kind_cluster: &Option<Option<String>>) {
    helpers::ensure_program_installed("docker");
    helpers::ensure_program_installed("kind");

    if let Some(kind_cluster) = kind_cluster {
        match kind_cluster {
            Some(kind_cluster_nane) => create_cluster_if_not_exists(kind_cluster_nane),
            None => create_cluster_if_not_exists(DEFAULT_KIND_CLUSTER_NAME),
        }
    }
}

fn create_cluster_if_not_exists(name: &str) {
    if check_if_kind_cluster_exists(name) {
        warn!("The kind cluster {name} is already running, not re-creating it. Use `kind delete cluster --name {name}` to delete it");
    } else {
        info!("Creating kind cluster {name}");
        helpers::execute_command_with_stdin(
            vec!["kind", "create", "cluster", "--name", name, "--config", "-"],
            KIND_CLUSTER_DEFINITION,
        );
    }
}

fn check_if_kind_cluster_exists(name: &str) -> bool {
    let result = helpers::execute_command(vec!["kind", "get", "clusters"]);
    result.lines().any(|cluster_name| cluster_name == name)
}
