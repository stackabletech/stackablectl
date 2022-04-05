use clap::Parser;
use std::str::FromStr;

#[derive(Parser)]
#[clap(author, version, about)]
pub struct CliArgs {
    #[clap(subcommand)]
    pub cmd: CliCommand,
}

#[derive(Parser)]
pub enum CliCommand {
    Deploy(DeployCommand),
}

#[derive(Parser)]
pub struct DeployCommand {
    /// Stackable operators to install with optional version specification.
    /// Must have the form "name[=version]" e.g. "superset=0.3.0".
    /// If you want to deploy multiple operators you have to use the -o flag multiple times.
    #[clap(short, long, multiple_occurrences(true))]
    pub operator: Vec<Operator>,

    /// When enabled we'll automatically create a 4 node kind cluster. "
    /// If this was provided with no argument the kind cluster will have the name default name "stackable-demo"
    /// Otherwise the provided name will be used
    #[clap(short, long)]
    pub kind: bool,

    /// The name of the created kind cluster when `--kind` is used
    #[clap(long, default_value = "stackable-platform")]
    pub kind_cluster_name: String,
}

#[derive(Debug)]
pub struct Operator {
    operator_name: String,
    version: Option<String>,
    example: Option<String>,
}

impl Operator {
    fn new(operator_name: String, version: Option<String>) -> Result<Self, String> {
        if crate::VALID_OPERATORS_WITH_EXAMPLES.contains_key(&operator_name) {
            Ok(Operator {
                operator_name,
                version,
                example: None, // TODO
            })
        } else {
            Err(format!("The operator {operator_name} does not exist or stackablectl is to old to know the operator"))
        }
    }
}

impl FromStr for Operator {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts = s.split('=').collect::<Vec<_>>();
        match parts[..] {
            [operator] => Operator::new(operator.to_string(), None),
            [operator, version] => Operator::new(operator.to_string(), Some(version.to_string())),
            _ => Err(format!("Could not parse the operator definition {s}")),
        }
    }
}
