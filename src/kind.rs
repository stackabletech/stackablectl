use std::error::Error;

use crate::helpers;
use log::{info, warn};

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

pub fn handle_cli_arguments(
    kind_cluster: bool,
    kind_cluster_name: &str,
) -> Result<(), Box<dyn Error>> {
    if kind_cluster {
        helpers::ensure_program_installed("docker")?;
        helpers::ensure_program_installed("kind")?;

        create_cluster_if_not_exists(kind_cluster_name)?;
    }

    Ok(())
}

fn create_cluster_if_not_exists(name: &str) -> Result<(), Box<dyn Error>> {
    if check_if_kind_cluster_exists(name)? {
        warn!("The kind cluster {name} is already running, not re-creating it. Use `kind delete cluster --name {name}` to delete it");
    } else {
        info!("Creating kind cluster {name}");
        helpers::execute_command_with_stdin(
            vec!["kind", "create", "cluster", "--name", name, "--config", "-"],
            KIND_CLUSTER_DEFINITION,
        )?;
    }

    Ok(())
}

fn check_if_kind_cluster_exists(name: &str) -> Result<bool, Box<dyn Error>> {
    let result = helpers::execute_command(vec!["kind", "get", "clusters"])?;
    Ok(result.lines().any(|cluster_name| cluster_name == name))
}
