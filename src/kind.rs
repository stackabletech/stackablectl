use crate::helpers;
use log::warn;

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

pub fn create_cluster(name: &str) {
    if check_if_kind_cluster_exists(name) {
        warn!("The kind cluster {name} is already running, not re-creating it");
    } else {
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
