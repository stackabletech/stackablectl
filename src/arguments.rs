use crate::operator::CliCommandOperator;
use crate::release::CliCommandRelease;
use crate::stack::CliCommandStack;
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

    /// f you don't have access to the Stackable Helm repos you can mirror the repo and provide the URL here
    /// (e.g. https://my.repo/repository/stackable-stable/).
    #[clap(
        long,
        default_value = "https://repo.stackable.tech/repository/helm-stable"
    )]
    pub helm_repo_stackable_stable: String,

    /// f you don't have access to the Stackable Helm repos you can mirror the repo and provide the URL here
    /// (e.g. https://my.repo/repository/stackable-test/).
    #[clap(
        long,
        default_value = "https://repo.stackable.tech/repository/helm-test"
    )]
    pub helm_repo_stackable_test: String,

    /// f you don't have access to the Stackable Helm repos you can mirror the repo and provide the URL here
    /// (e.g. https://my.repo/repository/stackable-dev/).
    #[clap(
        long,
        default_value = "https://repo.stackable.tech/repository/helm-dev"
    )]
    pub helm_repo_stackable_dev: String,
}

#[derive(Parser)]
pub enum CliCommand {
    /// This subcommand interacts with single operators if you don’t want to install the full platform.
    #[clap(subcommand, alias("o"), alias("op"))]
    Operator(CliCommandOperator),

    /// This subcommand interacts with all operators of the platform that are released together.
    #[clap(subcommand, alias("r"), alias("re"))]
    Release(CliCommandRelease),

    /// This subcommand interacts with stacks, which are ready-to-use combinations of products.
    #[clap(subcommand, alias("s"), alias("st"))]
    Stack(CliCommandStack),
}

#[derive(Clone, Parser, ArgEnum)]
pub enum OutputType {
    Text,
    Json,
    Yaml,
}
