use crate::helpers;
use std::io::Write;
use std::process::{Command, Stdio};

pub fn start(cluster_name: &str) {
    helpers::ensure_program_installed("docker");
    helpers::ensure_program_installed("kind");

    let child = Command::new("kind")
        .args(["create", "cluster", "--name", cluster_name, "--config", "-"])
        .stdin(Stdio::piped())
        .spawn()
        .expect("Failed to spawn kind command");

    child
        .stdin
        .as_ref()
        .unwrap()
        .write_all(KIND_CLUSTER_DEFINITION.as_bytes())
        .expect("failed to write kind cluster definition via stdin");
    if !child.wait_with_output().unwrap().status.success() {
        panic!("Failed to create kind cluster, see kind logs");
    }
}

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
