use crate::{
    demo::CliCommandDemo, operator::CliCommandOperator, release::CliCommandRelease,
    services::CliCommandServices, stack::CliCommandStack,
};
use clap::{ArgEnum, Parser};
use log::LevelFilter;

#[derive(Parser)]
#[clap(author, version, about)]
pub struct CliArgs {
    #[clap(subcommand)]
    pub cmd: CliCommand,

    /// Log level. One of Error, Warn, Info, Debug or Trace
    #[clap(short, long, default_value = "Info")]
    pub log_level: LevelFilter,

    /// Namespace where to deploy the products and operators
    #[clap(short, long, default_value = "default")]
    pub namespace: String,

    /// If you don't have access to the Stackable Helm repos you can mirror the repo and provide the URL here
    /// (e.g. <https://my.repo/repository/stackable-stable/>).
    #[clap(
        long,
        default_value = "https://repo.stackable.tech/repository/helm-stable"
    )]
    pub helm_repo_stackable_stable: String,

    /// If you don't have access to the Stackable Helm repos you can mirror the repo and provide the URL here
    /// (e.g. <https://my.repo/repository/stackable-test/>).
    #[clap(
        long,
        default_value = "https://repo.stackable.tech/repository/helm-test"
    )]
    pub helm_repo_stackable_test: String,

    /// If you don't have access to the Stackable Helm repos you can mirror the repo and provide the URL here
    /// (e.g. <https://my.repo/repository/stackable-dev/>).
    #[clap(
        long,
        default_value = "https://repo.stackable.tech/repository/helm-dev"
    )]
    pub helm_repo_stackable_dev: String,

    /// If you don't have access to the Stackable GitHub repos or you want to maintain your own releases you can specify additional YAML files containing release information.
    /// Have a look at <https://raw.githubusercontent.com/stackabletech/release/main/releases.yaml> for the structure.
    /// Can either be an URL or a path to a file e.g. `https://my.server/my-releases.yaml`, '/etc/my-releases.yaml' or `C:\Users\Sebastian\my-releases.yaml`.
    /// Can be specified multiple times.
    #[clap(long, multiple_occurrences(true))]
    pub additional_release_files: Vec<String>,

    /// If you don't have access to the Stackable GitHub repos or you want to maintain your own stacks you can specify additional YAML files containing stack information.
    /// Have a look at <https://raw.githubusercontent.com/stackabletech/stackablectl/main/stacks.yaml> for the structure.
    /// Can either be an URL or a path to a file e.g. `https://my.server/my-stacks.yaml`, '/etc/my-stacks.yaml' or `C:\Users\Sebastian\my-stacks.yaml`.
    /// Can be specified multiple times.
    #[clap(long, multiple_occurrences(true))]
    pub additional_stack_files: Vec<String>,

    /// If you don't have access to the Stackable GitHub repos or you want to maintain your own demos you can specify additional YAML files containing demo information.
    /// Have a look at <https://raw.githubusercontent.com/stackabletech/stackablectl/main/demos.yaml> for the structure.
    /// Can either be an URL or a path to a file e.g. `https://my.server/my-demos.yaml`, '/etc/my-demos.yaml' or `C:\Users\Sebastian\my-demos.yaml`.
    /// Can be specified multiple times.
    #[clap(long, multiple_occurrences(true))]
    pub additional_demo_files: Vec<String>,
}

#[derive(Parser)]
pub enum CliCommand {
    /// This command interacts with demos, which are end-to-end demonstrations of the usage of the Stackable Data Platform.
    #[clap(subcommand, alias("d"), alias("de"))]
    Demo(CliCommandDemo),

    /// This command interacts with single operators if you don’t want to install the full platform.
    #[clap(subcommand, alias("o"), alias("op"))]
    Operator(CliCommandOperator),

    /// This command interacts with all operators of the platform that are released together.
    #[clap(subcommand, alias("r"), alias("re"))]
    Release(CliCommandRelease),

    /// This command interacts with stacks, which are ready-to-use combinations of products.
    #[clap(subcommand, alias("s"), alias("st"))]
    Stack(CliCommandStack),

    /// This command interacts with deployed services of products.
    #[clap(subcommand, alias("svc"))]
    Services(CliCommandServices),
}

#[derive(Clone, Parser, ArgEnum)]
pub enum OutputType {
    Text,
    Json,
    Yaml,
}
