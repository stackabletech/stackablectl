use std::sync::Mutex;

use crate::arguments::CliCommand;
use arguments::CliArgs;
use clap::Parser;
use lazy_static::lazy_static;
use log::info;

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

lazy_static! {
    pub static ref NAMESPACE: Mutex<String> = Mutex::new(String::new());
}

fn main() {
    let args = CliArgs::parse();
    env_logger::builder()
        .format_timestamp(None)
        .format_target(false)
        .filter_level(args.log_level)
        .init();

    let namespace = &args.namespace;
    if namespace != "default" {
        info!("Deploying into non-default namespace.\
            Please make sure not to deploy the same operator multiple times in different namespaces unless you know what you are doing (TM).");
    }
    *(NAMESPACE.lock().unwrap()) = namespace.to_string();

    helm::handle_common_cli_args(&args);
    release::handle_common_cli_args(&args);
    stack::handle_common_cli_args(&args);

    match &args.cmd {
        CliCommand::Operator(command) => command.handle(),
        CliCommand::Release(command) => command.handle(),
        CliCommand::Stack(command) => command.handle(),
    }
}
