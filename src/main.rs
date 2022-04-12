use crate::arguments::CliCommand;
use arguments::CliArgs;
use clap::Parser;
use log::info;
use phf::phf_map;

mod arguments;
mod helm;
mod helpers;
mod kind;
mod operator;
mod tooling;

/// key: Operator name
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
    env_logger::builder()
        .format_timestamp(None)
        .format_target(false)
        .filter_level(args.log_level)
        .init();

    match &args.cmd {
        CliCommand::CreateKindCluster(command) => {
            helpers::ensure_program_installed("docker");
            helpers::ensure_program_installed("kind");

            kind::create_cluster(&command.name);
        }
        CliCommand::DeployOperators(command) => {
            helpers::ensure_program_installed("kubectl");

            if command.operator.is_empty() {
                info!("No operators to install, specify them via the option `-o` or `--operator`");
            }
            for operator in &command.operator {
                operator.deploy(command.examples);
            }
        }
        CliCommand::DeployTooling(command) => {
            helpers::ensure_program_installed("kubectl");

            tooling::deploy(command);
        }
        CliCommand::AccessServices => {
            todo!("Must implement access-services command");
        }
    }
}
