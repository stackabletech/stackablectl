use crate::{operator::CliCommandOperator, release::CliCommandRelease, stack::CliCommandStack};
use clap::{ArgEnum, Command, Parser, ValueHint};
use clap_complete::{generate, Generator, Shell};
use log::LevelFilter;
use std::{io};

#[derive(Parser)]
#[clap(author, version, about)]
pub struct CliArgs {
    #[clap(subcommand)]
    pub cmd: CliCommand,

    /// Log level.
    #[clap(short, long, arg_enum, default_value = "info")]
    pub log_level: LogLevel,

    /// Namespace where to deploy the products and operators
    #[clap(short, long, default_value = "default", value_hint = ValueHint::Other)]
    pub namespace: String,

    /// If you don't have access to the Stackable Helm repos you can mirror the repo and provide the URL here
    /// (e.g. <https://my.repo/repository/stackable-stable/>).
    #[clap(
        long,
        default_value = "https://repo.stackable.tech/repository/helm-stable",
        value_hint = ValueHint::Url,
    )]
    pub helm_repo_stackable_stable: String,

    /// If you don't have access to the Stackable Helm repos you can mirror the repo and provide the URL here
    /// (e.g. <https://my.repo/repository/stackable-test/>).
    #[clap(
        long,
        default_value = "https://repo.stackable.tech/repository/helm-test",
        value_hint = ValueHint::Url,
    )]
    pub helm_repo_stackable_test: String,

    /// If you don't have access to the Stackable Helm repos you can mirror the repo and provide the URL here
    /// (e.g. <https://my.repo/repository/stackable-dev/>).
    #[clap(
        long,
        default_value = "https://repo.stackable.tech/repository/helm-dev",
        value_hint = ValueHint::Url,
    )]
    pub helm_repo_stackable_dev: String,

    /// If you don't have access to the Stackable GitHub repos or you want to maintain your own releases you can specify additional YAML files containing release information.
    /// Have a look [here](https://raw.githubusercontent.com/stackabletech/stackablectl/main/releases.yaml) for the structure.
    /// Can either be an URL or a path to a file e.g. `https://my.server/my-releases.yaml` or '/etc/my-releases.yaml' or `C:\Users\Sebastian\my-releases.yaml`.
    /// Can be specified multiple times.
    #[clap(long, multiple_occurrences(true), value_hint = ValueHint::FilePath)]
    pub additional_release_files: Vec<String>,

    /// If you don't have access to the Stackable GitHub repos or you want to maintain your own stacks you can specify additional YAML files containing stack information.
    /// Have a look [here](https://raw.githubusercontent.com/stackabletech/stackablectl/main/stacks.yaml) for the structure.
    /// Can either be an URL or a path to a file e.g. `https://my.server/my-stacks.yaml` or '/etc/my-stacks.yaml' or `C:\Users\Sebastian\my-stacks.yaml`.
    /// Can be specified multiple times.
    #[clap(long, multiple_occurrences(true), value_hint = ValueHint::FilePath)]
    pub additional_stack_files: Vec<String>,
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

    /// Output shell completion code for the specified shell.
    Completion(CliCommandCompletion),
}

#[derive(Clone, Parser, ArgEnum)]
pub enum OutputType {
    Text,
    Json,
    Yaml,
}

#[derive(Clone, Parser, Debug, ArgEnum)]
pub enum LogLevel {
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl From<&LogLevel> for LevelFilter {
    fn from(val: &LogLevel) -> Self {
        match val {
            LogLevel::Error => LevelFilter::Error,
            LogLevel::Warn => LevelFilter::Warn,
            LogLevel::Info => LevelFilter::Info,
            LogLevel::Debug => LevelFilter::Debug,
            LogLevel::Trace => LevelFilter::Trace,
        }
    }
}

#[derive(Parser)]
pub struct CliCommandCompletion {
    // Shell to generate the completions for
    #[clap(arg_enum, value_parser)]
    pub shell: Shell,
}

pub fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
    generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}
