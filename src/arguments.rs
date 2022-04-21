use crate::operator::CliCommandOperator;
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
}

#[derive(Parser)]
pub enum CliCommand {
    /// This subcommand interacts with single operators if you don’t want to install the full platform.
    #[clap(subcommand, alias("op"))]
    Operator(CliCommandOperator),
}

#[derive(Clone, Parser, ArgEnum)]
pub enum OutputType {
    Text,
    Json,
    Yaml,
}
