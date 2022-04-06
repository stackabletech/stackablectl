use crate::helpers;
use log::warn;
use std::io::Write;
use std::process::{Command, Stdio};

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
        let child = Command::new("kind")
            .args(["create", "cluster", "--name", name, "--config", "-"])
            .stdin(Stdio::piped())
            .spawn()
            .expect("Failed to spawn kind command");

        child
            .stdin
            .as_ref()
            .unwrap()
            .write_all(KIND_CLUSTER_DEFINITION.as_bytes())
            .expect("Failed to write kind cluster definition via stdin");
        if !child.wait_with_output().unwrap().status.success() {
            panic!("Failed to create kind cluster, see kind logs");
        }
    }
}

fn check_if_kind_cluster_exists(name: &str) -> bool {
    let result = helpers::execute_command(vec!["kind", "get", "clusters"]);
    result.lines().any(|cluster_name| cluster_name == name)
}
