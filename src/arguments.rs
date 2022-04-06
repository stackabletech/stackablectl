use crate::operator::Operator;
use clap::Parser;
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
    Deploy(DeployCommand),
}

#[derive(Parser)]
pub struct DeployCommand {
    /// Stackable operators to install with optional version specification.
    /// Must have the form `name[=version]` e.g. `superset=0.3.0`, `superset=0.3.0-nightly` or `superset=0.3.0-pr123`.
    /// If you want to deploy multiple operators you have to use the -o flag multiple times.
    #[clap(short, long, multiple_occurrences(true))]
    pub operator: Vec<Operator>,

    /// Will install the `simple` examples for the operators specified via `--operator` or `-o`
    #[clap(short, long)]
    pub examples: bool,

    /// When enabled we'll automatically create a 4 node kind cluster.
    /// If this was provided with no argument the kind cluster will have the name default name "stackable-demo".
    /// Otherwise the provided name will be used.
    #[clap(short, long)]
    pub kind: bool,

    /// The name of the created kind cluster when `--kind` is used.
    #[clap(long, default_value = "stackable-platform")]
    pub kind_cluster_name: String,
}
