use crate::arguments::CliCommand;
use arguments::CliArgs;
use clap::Parser;

mod arguments;
mod helm;
mod helpers;
mod kind;
mod kube;
mod operator;
mod release;
mod stack;

const AVAILABLE_OPERATORS: &[&str] = &[
    "airflow",
    "commons",
    "druid",
    "hbase",
    "hdfs",
    "hive",
    "kafka",
    "nifi",
    "opa",
    "secret",
    "spark",
    "spark-k8s",
    "superset",
    "trino",
    "zookeeper",
    // Deprecated
    "regorule",
    "monitoring",
];

fn main() {
    let args = CliArgs::parse();
    env_logger::builder()
        .format_timestamp(None)
        .format_target(false)
        .filter_level(args.log_level)
        .init();
    helm::handle_common_cli_args(&args);
    release::handle_common_cli_args(&args);

    match &args.cmd {
        CliCommand::Operator(command) => command.handle(),
        CliCommand::Release(command) => command.handle(),
        CliCommand::Stack(command) => command.handle(),
    }
}
