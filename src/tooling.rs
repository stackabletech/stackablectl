use crate::arguments::DeployToolingCommand;
use crate::helm::HELM_PROMETHEUS_REPO_URL;
use crate::{helm, helpers};
use log::info;

const PROMETHEUS_SCRAPE_CONFIG: &str = r#"
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: scrape-label
  labels:
    release: prometheus-operator
spec:
  endpoints:
  - port: metrics
  jobLabel: app.kubernetes.io/instance
  selector:
    matchLabels:
      prometheus.io/scrape: "true"
  namespaceSelector:
    any: true
"#;

pub fn deploy(command: &DeployToolingCommand) {
    match command {
        DeployToolingCommand::Minio => {
            todo!("Must implement MinIO")
        }
        DeployToolingCommand::Prometheus => {
            info!("Installing Prometheus");
            helm::install_helm_release("prometheus-operator", HELM_PROMETHEUS_REPO_URL, vec![]);
            info!("Installing Prometheus scrape configuration");
            helpers::execute_command_with_stdin(
                vec!["kubectl", "apply", "-f", "-"],
                PROMETHEUS_SCRAPE_CONFIG,
            );
        }
    }
}
