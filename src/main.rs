use crate::arguments::CliCommand;
use arguments::CliArgs;
use clap::{CommandFactory, Parser};
use lazy_static::lazy_static;
use log::error;
use std::{error::Error, process::exit, sync::Mutex};

mod arguments;
mod demo;
mod helm;
mod helpers;
mod kind;
mod kube;
mod operator;
mod release;
mod services;
mod stack;

const AVAILABLE_OPERATORS: &[&str] = &[
    "airflow",
    "commons",
    "druid",
    "hbase",
    "hdfs",
    "hive",
    "kafka",
    "listener",
    "nifi",
    "opa",
    "secret",
    "spark-k8s",
    "superset",
    "trino",
    "zookeeper",
];

lazy_static! {
    pub static ref NAMESPACE: Mutex<String> = Mutex::new(String::new());
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = CliArgs::parse();
    env_logger::builder()
        .format_timestamp(None)
        .format_target(false)
        .filter_level(args.log_level.into())
        .init();

    let namespace = &args.namespace;
    *(NAMESPACE.lock()?) = namespace.to_string();

    helm::handle_common_cli_args(&args);
    release::handle_common_cli_args(&args);
    stack::handle_common_cli_args(&args);
    demo::handle_common_cli_args(&args);

    let result = match &args.cmd {
        CliCommand::Operator(command) => command.handle().await,
        CliCommand::Release(command) => command.handle().await,
        CliCommand::Stack(command) => command.handle().await,
        CliCommand::Demo(command) => command.handle().await,
        CliCommand::Services(command) => command.handle().await,
        CliCommand::Completion(command) => {
            let mut cmd = CliArgs::command();
            arguments::print_completions(command.shell, &mut cmd);
            Ok(())
        }
    };

    if let Err(err) = &result {
        error!("{err}");
        exit(-1);
    }

    result
}
