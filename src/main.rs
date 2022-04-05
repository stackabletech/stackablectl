extern crate core;

use crate::arguments::CliCommand;
use arguments::CliArgs;
use clap::Parser;
use phf::phf_map;

mod arguments;
mod helpers;
mod kind;

/// key: Operator Name
/// value: Optional example file
// const STACKABLE_HELM_REPOS: phf::Map<&'static str, &'static str> = phf_map! {
//     "stackable-dev" => "https://repo.stackable.tech/repository/helm-dev",
//     "stackable-test" => "https://repo.stackable.tech/repository/helm-test",
//     "stackable" => "https://repo.stackable.tech/repository/helm-stable",
//     "prometheus-community" => "https://prometheus-community.github.io/helm-charts",
// };

// HELM_DEV_REPO_NAME = "stackable-dev"
// HELM_DEV_REPO_URL = "https://repo.stackable.tech/repository/helm-dev"
// HELM_TEST_REPO_NAME = "stackable-test"
// HELM_TEST_REPO_URL = "https://repo.stackable.tech/repository/helm-test"
// HELM_STABLE_REPO_NAME = "stackable"
// HELM_STABLE_REPO_URL = "https://repo.stackable.tech/repository/helm-stable"
// HELM_PROMETHEUS_REPO_NAME = "prometheus-community"
// HELM_PROMETHEUS_REPO_URL = "https://prometheus-community.github.io/helm-charts"
// HELM_PROMETHEUS_CHART_NAME = "kube-prometheus-stack"

/// key: Operator Name
/// value: Optional example file
const VALID_OPERATORS_WITH_EXAMPLES: phf::Map<&'static str, Option<&'static str>> = phf_map! {
    "airflow" => Some("https://raw.githubusercontent.com/stackabletech/airflow-operator/main/examples/simple-airflow-cluster.yaml"),
    "commons" => None,
    "druid" => Some("https://raw.githubusercontent.com/stackabletech/druid-operator/main/examples/simple-druid-cluster.yaml"),
    "hbase" => Some("https://raw.githubusercontent.com/stackabletech/hbase-operator/main/examples/simple-hbase-cluster.yaml"),
    "hdfs" => Some("https://raw.githubusercontent.com/stackabletech/hdfs-operator/main/examples/simple-hdfs-cluster.yaml"),
    "hive" => Some("https://raw.githubusercontent.com/stackabletech/hive-operator/main/examples/simple-hive-cluster.yaml"),
    "kafka" => Some("https://raw.githubusercontent.com/stackabletech/kafka-operator/main/examples/simple-kafka-cluster.yaml"),
    "nifi" => Some("https://raw.githubusercontent.com/stackabletech/nifi-operator/main/examples/simple-nifi-cluster.yaml"),
    "opa" => Some("https://raw.githubusercontent.com/stackabletech/opa-operator/main/examples/simple-opa-cluster.yaml"),
    "secret" => None,
    "spark" => Some("https://raw.githubusercontent.com/stackabletech/spark-operator/main/examples/simple-spark-cluster.yaml"),
    "spark-k8s" => None, // TODO
    "superset" => Some("https://raw.githubusercontent.com/stackabletech/superset-operator/main/examples/simple-superset-cluster.yaml"),
    "trino" => Some("https://raw.githubusercontent.com/stackabletech/trino-operator/main/examples/simple-trino-cluster.yaml"),
    "zookeeper" => Some("https://raw.githubusercontent.com/stackabletech/zookeeper-operator/main/examples/simple-zookeeper-cluster.yaml"),
};

fn main() {
    let args = CliArgs::parse();

    match &args.cmd {
        CliCommand::Deploy(deploy_command) => {
            if deploy_command.kind {
                kind::start(&deploy_command.kind_cluster_name);
            }

            println!("TODO: Starting deploying operators");
        }
    }
}
