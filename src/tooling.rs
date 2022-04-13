use crate::arguments::DeployToolingCommand;
use crate::{helm, helpers, kube};
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
            helm::install_helm_release_from_repo(
                "prometheus-operator",
                "prometheus-community",
                "prometheus-operator",
                None,
            );
            info!("Installing Prometheus scrape configuration");
            kube::deploy_manifest(PROMETHEUS_SCRAPE_CONFIG);
        }
    }
}
