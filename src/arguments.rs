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
    /// Create a local kubernetes cluster for testing purposes
    ///
    /// Kind is a tool to spin up a local kubernetes cluster running on docker on your machine.
    /// This scripts creates such a cluster consisting of 4 nodes to test the Stackable Data Platform.
    ///
    /// You need to have `docker` and `kind` installed. Have a look at the README at https://github.com/stackabletech/stackablectl on how to install them
    CreateKindCluster(CreateKindClusterCommand),
    /// Deploy product operators
    ///
    /// You need to have `kubectl` and `helm` installed. Have a look at the README at https://github.com/stackabletech/stackablectl on how to install them
    DeployOperators(DeployOperatorsCommand),
    /// Deploy additional tooling like S3 data storage or Monitoring
    ///
    /// You need to have `kubectl` and `helm` installed. Have a look at the README at https://github.com/stackabletech/stackablectl on how to install them
    #[clap(subcommand)]
    DeployTooling(DeployToolingCommand),
    /// Access deployed services
    ///
    /// You need to have `kubectl` installed. Have a look at the README at https://github.com/stackabletech/stackablectl on how to install them
    AccessServices,
}

#[derive(Parser)]
pub struct CreateKindClusterCommand {
    /// The name of the kind cluster to create
    #[clap(short, long, default_value = "stackable-data-platform")]
    pub name: String,
}

#[derive(Parser)]
pub struct DeployOperatorsCommand {
    /// Stackable operators to install with optional version specification.
    /// Must have the form `name[=version]` e.g. `superset`, `superset=0.3.0`, `superset=0.3.0-nightly` or `superset=0.3.0-pr123`.
    /// If you want to deploy multiple operators you have to use the -o flag multiple times.
    #[clap(short, long, multiple_occurrences(true))]
    pub operator: Vec<Operator>,

    /// Will install the `simple` examples for the operators specified via `--operator` or `-o`
    #[clap(short, long)]
    pub examples: bool,
}

#[derive(Parser)]
pub enum DeployToolingCommand {
    /// S3 storage provider
    Minio,
    /// Monitoring with Prometheus and Grafana
    Prometheus,
}
