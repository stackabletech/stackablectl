use std::{collections::HashMap, sync::Arc};

use clap::{Args, Subcommand};
use comfy_table::{
    ContentArrangement, Table,
    presets::{NOTHING, UTF8_FULL},
};
use indexmap::IndexMap;
use semver::Version;
use serde::Serialize;
use snafu::{ResultExt, Snafu};
use stackable_cockpit::{
    constants::{
        DEFAULT_OPERATOR_NAMESPACE, HELM_REPO_NAME_DEV, HELM_REPO_NAME_STABLE, HELM_REPO_NAME_TEST,
    },
    helm::{self, Release},
    oci,
    platform::{
        namespace,
        operator::{self, ChartSourceType},
    },
    utils::{
        self,
        chartsource::ChartSourceMetadata,
        k8s::{self, Client},
        path::PathOrUrlParseError,
        yaml::values_for_operator,
    },
    xfer,
};
use tracing::{Span, debug, info, instrument};
use tracing_indicatif::{indicatif_println, span_ext::IndicatifSpanExt};

use crate::{
    args::{CommonClusterArgs, CommonClusterArgsError},
    cli::{Cli, OutputType},
    utils::{InvalidRepoNameError, helm_repo_name_to_repo_url, load_operator_values},
};

const INSTALL_AFTER_HELP_TEXT: &str = "Examples:

Use \"stackablectl operator install <OPERATOR> -c <OPTION>\" to create a local cluster";

#[derive(Debug, Args)]
pub struct OperatorArgs {
    #[command(subcommand)]
    subcommand: OperatorCommands,
}

#[derive(Debug, Subcommand)]
pub enum OperatorCommands {
    /// List available operators
    #[command(alias("ls"))]
    List(OperatorListArgs),

    /// Print out detailed operator information
    #[command(alias("desc"))]
    Describe(OperatorDescribeArgs),

    /// Install one or more operators
    #[command(aliases(["i", "in"]), after_help = INSTALL_AFTER_HELP_TEXT)]
    Install(OperatorInstallArgs),

    /// Uninstall one or more operators
    #[command(aliases(["rm", "un"]))]
    Uninstall(OperatorUninstallArgs),

    /// List installed operators
    Installed(OperatorInstalledArgs),
}

#[derive(Debug, Args)]
pub struct OperatorListArgs {
    #[arg(short, long = "output", value_enum, default_value_t = Default::default())]
    output_type: OutputType,
}

#[derive(Debug, Args)]
pub struct OperatorDescribeArgs {
    /// Operator to describe
    #[arg(name = "OPERATOR", required = true)]
    operator_name: String,

    #[arg(short, long = "output", value_enum, default_value_t = Default::default())]
    output_type: OutputType,
}

#[derive(Debug, Args)]
pub struct OperatorInstallArgs {
    /// Operator(s) to install
    #[arg(name = "OPERATORS", required = true)]
    #[arg(long_help = "Operator(s) to install (space separated)

Each must have the form 'name[=version]'.
If no version is specified the latest nightly version - built from the main branch - will be used.
Possible valid values are:

- superset (equivalent to superset=0.0.0-dev)
- superset=25.3.0
- superset=0.0.0-pr123 (Pull Request build)

Use \"stackablectl operator list\" to list available versions for all operators
Use \"stackablectl operator describe <OPERATOR>\" to get available versions for one operator")]
    operators: Vec<coffee::OperatorOrCoffee>,

    /// Namespace in the cluster used to deploy the operators
    #[arg(long, default_value = DEFAULT_OPERATOR_NAMESPACE, visible_aliases(["operator-ns"]))]
    pub operator_namespace: String,

    #[command(flatten)]
    local_cluster: CommonClusterArgs,
}

#[derive(Debug, Args)]
pub struct OperatorUninstallArgs {
    /// One or more operators to uninstall
    #[arg(required = true)]
    operators: Vec<operator::OperatorSpec>,

    /// Namespace in the cluster used to deploy the operators
    #[arg(long, default_value = DEFAULT_OPERATOR_NAMESPACE, visible_aliases(["operator-ns"]))]
    pub operator_namespace: String,
}

#[derive(Debug, Args)]
pub struct OperatorInstalledArgs {
    #[arg(short, long = "output", value_enum, default_value_t = Default::default())]
    output_type: OutputType,

    /// Namespace in the cluster used to deploy the operators
    #[arg(long, default_value = DEFAULT_OPERATOR_NAMESPACE, visible_aliases(["operator-ns"]))]
    pub operator_namespace: String,
}

#[derive(Debug, Snafu)]
pub enum CmdError {
    #[snafu(display("invalid repository name"))]
    InvalidRepoName { source: InvalidRepoNameError },

    #[snafu(display("invalid semantic helm chart version {version:?}"))]
    InvalidHelmChartVersion {
        source: semver::Error,
        version: String,
    },

    #[snafu(display("unknown repository name {name:?}"))]
    UnknownRepoName { name: String },

    #[snafu(display("Helm error"))]
    HelmError { source: helm::Error },

    #[snafu(display("cluster argument error"))]
    CommonClusterArgs { source: CommonClusterArgsError },

    #[snafu(display("failed to serialize YAML output"))]
    SerializeYamlOutput { source: serde_yaml::Error },

    #[snafu(display("failed to serialize JSON output"))]
    SerializeJsonOutput { source: serde_json::Error },

    #[snafu(display("failed to create Kubernetes client"))]
    KubeClientCreate {
        #[snafu(source(from(k8s::Error, Box::new)))]
        source: Box<k8s::Error>,
    },

    #[snafu(display("failed to create namespace {namespace:?}"))]
    NamespaceCreate {
        source: namespace::Error,
        namespace: String,
    },

    #[snafu(display("OCI error"))]
    OciError { source: oci::Error },

    #[snafu(display("path/url parse error"))]
    PathOrUrlParse { source: PathOrUrlParseError },

    #[snafu(display("failed to load operator values"))]
    LoadOperatorValues { source: crate::utils::Error },
}

/// This list contains a list of operator version grouped by stable, test and
/// dev lines. The lines can be accessed by the globally defined constants like
/// [`HELM_REPO_NAME_STABLE`].
#[derive(Debug, Serialize)]
pub struct OperatorVersionList(HashMap<String, Vec<String>>);

impl OperatorArgs {
    pub async fn run(
        &self,
        cli: &Cli,
        transfer_client: Arc<xfer::Client>,
    ) -> Result<String, CmdError> {
        match &self.subcommand {
            OperatorCommands::List(args) => list_cmd(args, cli).await,
            OperatorCommands::Describe(args) => describe_cmd(args, cli).await,
            OperatorCommands::Install(args) => install_cmd(args, cli, transfer_client).await,
            OperatorCommands::Uninstall(args) => uninstall_cmd(args),
            OperatorCommands::Installed(args) => installed_cmd(args),
        }
    }
}

#[instrument(skip_all, fields(indicatif.pb_show = true))]
async fn list_cmd(args: &OperatorListArgs, cli: &Cli) -> Result<String, CmdError> {
    debug!("Listing operators");
    Span::current().pb_set_message("Fetching operator information");

    // Build map which maps artifacts to a chart source
    let source_index_files =
        build_source_index_file_list(&ChartSourceType::from(cli.chart_type())).await?;

    // Iterate over all valid operators and create a list of versions grouped
    // by stable, test and dev lines
    let versions_list = build_versions_list(&source_index_files)?;

    match args.output_type {
        OutputType::Plain | OutputType::Table => {
            let (arrangement, preset) = match args.output_type {
                OutputType::Plain => (ContentArrangement::Disabled, NOTHING),
                _ => (ContentArrangement::Dynamic, UTF8_FULL),
            };

            let mut table = Table::new();
            table
                .set_header(vec!["#", "OPERATOR", "STABLE VERSIONS"])
                .set_content_arrangement(arrangement)
                .load_preset(preset);

            for (index, (operator_name, versions)) in versions_list.iter().enumerate() {
                let versions_string = match versions.0.get(HELM_REPO_NAME_STABLE) {
                    Some(v) => v.join(", "),
                    None => "".into(),
                };
                table.add_row(vec![
                    (index + 1).to_string(),
                    operator_name.clone(),
                    versions_string,
                ]);
            }

            let mut result = Cli::result();

            result
                .with_command_hint(
                    "stackablectl operator describe [OPTIONS] <OPERATOR>",
                    "display further information for the specified operator",
                )
                .with_command_hint(
                    "stackablectl operator install [OPTIONS] <OPERATORS>...",
                    "install one or more operators",
                )
                .with_output(table.to_string());

            Ok(result.render())
        }
        OutputType::Json => serde_json::to_string(&versions_list).context(SerializeJsonOutputSnafu),
        OutputType::Yaml => serde_yaml::to_string(&versions_list).context(SerializeYamlOutputSnafu),
    }
}

#[instrument(skip_all, fields(indicatif.pb_show = true))]
async fn describe_cmd(args: &OperatorDescribeArgs, cli: &Cli) -> Result<String, CmdError> {
    debug!(operator_name = %args.operator_name, "Describing operator");
    Span::current().pb_set_message("Fetching operator information");

    // Build map which maps artifacts to a chart source
    let source_index_files =
        build_source_index_file_list(&ChartSourceType::from(cli.chart_type())).await?;

    // Create a list of versions for this operator
    let versions_list = build_versions_list_for_operator(&args.operator_name, &source_index_files)?;

    match args.output_type {
        OutputType::Plain | OutputType::Table => {
            let arrangement = match args.output_type {
                OutputType::Plain => ContentArrangement::Disabled,
                _ => ContentArrangement::Dynamic,
            };

            let stable_versions_string = match versions_list.0.get(HELM_REPO_NAME_STABLE) {
                Some(v) => v.join(", "),
                None => "".into(),
            };

            let test_versions_string = match versions_list.0.get(HELM_REPO_NAME_TEST) {
                Some(v) => v.join(", "),
                None => "".into(),
            };

            let dev_versions_string = match versions_list.0.get(HELM_REPO_NAME_DEV) {
                Some(v) => v.join(", "),
                None => "".into(),
            };

            let mut table = Table::new();
            table
                .set_content_arrangement(arrangement)
                .load_preset(NOTHING)
                .add_row(vec!["OPERATOR", &args.operator_name.to_string()])
                .add_row(vec!["STABLE VERSIONS", stable_versions_string.as_str()])
                .add_row(vec!["TEST VERSIONS", test_versions_string.as_str()])
                .add_row(vec!["DEV VERSIONS", dev_versions_string.as_str()]);

            let mut result = Cli::result();

            result
                .with_command_hint(
                    format!(
                        "stackablectl operator install {operator_name}",
                        operator_name = args.operator_name
                    ),
                    "install the operator",
                )
                .with_command_hint("stackablectl operator list", "list all available operators")
                .with_output(table.to_string());

            Ok(result.render())
        }
        OutputType::Json => serde_json::to_string(&versions_list).context(SerializeJsonOutputSnafu),
        OutputType::Yaml => serde_yaml::to_string(&versions_list).context(SerializeYamlOutputSnafu),
    }
}

#[instrument(skip_all, fields(indicatif.pb_show = true))]
async fn install_cmd(
    args: &OperatorInstallArgs,
    cli: &Cli,
    transfer_client: Arc<xfer::Client>,
) -> Result<String, CmdError> {
    info!("Installing operator(s)");
    Span::current().pb_set_message("Installing operator(s)");

    let operators: Vec<&operator::OperatorSpec> = args
        .operators
        .iter()
        .filter_map(|operator| match operator {
            coffee::OperatorOrCoffee::Coffee => {
                indicatif_println!("{}", coffee::COFFEE_ASCII_ART);
                None
            }
            coffee::OperatorOrCoffee::Operator(spec) => Some(spec),
        })
        .collect();

    // In case no operators need to be installed (e.g. coffee was already installed), there is no
    // need to connect to Kubernetes and potentially produce error messages.
    if operators.is_empty() {
        return Ok(String::new());
    }

    args.local_cluster
        .install_if_needed()
        .await
        .context(CommonClusterArgsSnafu)?;

    let client = Client::new().await.context(KubeClientCreateSnafu)?;

    namespace::create_if_needed(&client, args.operator_namespace.clone())
        .await
        .context(NamespaceCreateSnafu {
            namespace: args.operator_namespace.clone(),
        })?;

    let values_file = cli.get_values_file().context(PathOrUrlParseSnafu)?;
    let operator_values = load_operator_values(values_file.as_ref(), &transfer_client)
        .await
        .context(LoadOperatorValuesSnafu)?;

    for operator in &operators {
        let operator_helm_values = values_for_operator(&operator_values, &operator.name);

        operator
            .install(
                &args.operator_namespace,
                &ChartSourceType::from(cli.chart_type()),
                &operator_helm_values,
            )
            .context(HelmSnafu)?;

        info!(%operator, "Installed operator");
        indicatif_println!("Installed {operator} operator");
    }

    let mut result = Cli::result();

    result
        .with_command_hint(
            "stackablectl operator installed [OPTIONS]",
            "list installed operators",
        )
        .with_output(format!(
            "Installed {num_of_operators} {suffix}",
            num_of_operators = operators.len(),
            suffix = if operators.len() == 1 {
                "operator"
            } else {
                "operators"
            }
        ));

    Ok(result.render())
}

#[instrument(skip_all, fields(indicatif.pb_show = true))]
fn uninstall_cmd(args: &OperatorUninstallArgs) -> Result<String, CmdError> {
    info!("Uninstalling operator(s)");
    Span::current().pb_set_message("Uninstalling operator(s)");

    for operator in &args.operators {
        operator
            .uninstall(&args.operator_namespace)
            .context(HelmSnafu)?;
    }

    let mut result = Cli::result();

    result
        .with_command_hint(
            "stackablectl operator installed [OPTIONS]",
            "list remaining installed operators",
        )
        .with_output(format!(
            "Uninstalled {num_of_operators} {suffix}",
            num_of_operators = args.operators.len(),
            suffix = if args.operators.len() == 1 {
                "operator"
            } else {
                "operators"
            }
        ));

    Ok(result.render())
}

#[instrument(skip_all, fields(indicatif.pb_show = true))]
fn installed_cmd(args: &OperatorInstalledArgs) -> Result<String, CmdError> {
    info!("Listing installed operators");
    Span::current().pb_set_message("Fetching operator information");

    type ReleaseList = IndexMap<String, Release>;

    let installed: ReleaseList = helm::list_releases(&args.operator_namespace)
        .context(HelmSnafu)?
        .into_iter()
        .filter(|release| {
            operator::VALID_OPERATORS
                .iter()
                .any(|valid| release.name == utils::operator_chart_name(valid))
        })
        .map(|release| (release.name.clone(), release))
        .collect();

    match args.output_type {
        OutputType::Plain | OutputType::Table => {
            if installed.is_empty() {
                return Ok("No installed operators".into());
            }

            let (arrangement, preset) = match args.output_type {
                OutputType::Plain => (ContentArrangement::Disabled, NOTHING),
                _ => (ContentArrangement::Dynamic, UTF8_FULL),
            };

            let mut table = Table::new();

            table
                .set_header(vec![
                    "OPERATOR",
                    "VERSION",
                    "NAMESPACE",
                    "STATUS",
                    "LAST UPDATED",
                ])
                .set_content_arrangement(arrangement)
                .load_preset(preset);

            for (release_name, release) in installed {
                table.add_row(vec![
                    release_name,
                    release.version,
                    release.namespace,
                    release.status,
                    release.last_updated,
                ]);
            }

            let mut result = Cli::result();

            result
                .with_command_hint(
                    "stackablectl operator install [OPTIONS] <OPERATORS>...",
                    "install one or more additional operators",
                )
                .with_command_hint(
                    "stackablectl operator uninstall [OPTIONS] <OPERATORS>...",
                    "uninstall one or more operators",
                )
                .with_output(table.to_string());

            Ok(result.render())
        }
        OutputType::Json => serde_json::to_string(&installed).context(SerializeJsonOutputSnafu),
        OutputType::Yaml => serde_yaml::to_string(&installed).context(SerializeYamlOutputSnafu),
    }
}

/// Builds a map which maps artifact tags to a chart source.
#[instrument]
async fn build_source_index_file_list<'a>(
    chart_source: &ChartSourceType,
) -> Result<HashMap<&'a str, ChartSourceMetadata>, CmdError> {
    debug!("Building source index file list");

    let mut source_index_files: HashMap<&str, ChartSourceMetadata> = HashMap::new();

    match chart_source {
        ChartSourceType::OCI => {
            source_index_files = oci::get_oci_index().await.context(OciSnafu)?;

            debug!(count = source_index_files.len(), "OCI Repository entries");

            // TODO (@NickLarsenNZ): Look into why this is so deeply nested with duplicate data.
            // source_index_files
            //     .iter()
            //     .for_each(|(&repo_name, chart_source_metadata)| {
            //         let x = chart_source_metadata.entries.len();
            //         tracing::trace!(repo_name, x, "thing");
            //         let _ = &chart_source_metadata
            //             .entries
            //             .iter()
            //             // y (below) is a Vec
            //             .for_each(|(x, y)| tracing::error!(x, "blah {:?}", y));
            //     });
        }
        ChartSourceType::Repo => {
            for helm_repo_name in [
                HELM_REPO_NAME_STABLE,
                HELM_REPO_NAME_TEST,
                HELM_REPO_NAME_DEV,
            ] {
                let helm_repo_url =
                    helm_repo_name_to_repo_url(helm_repo_name).context(InvalidRepoNameSnafu)?;

                source_index_files.insert(
                    helm_repo_name,
                    helm::get_helm_index(helm_repo_url)
                        .await
                        .context(HelmSnafu)?,
                );

                debug!("Helm Repository entries: {:?}", source_index_files);
            }
        }
    };

    Ok(source_index_files)
}

/// Iterates over all valid operators and creates a list of versions grouped
/// by stable, test and dev lines based on the list of Helm repo index files.
#[instrument(skip_all)]
fn build_versions_list(
    helm_index_files: &HashMap<&str, ChartSourceMetadata>,
) -> Result<IndexMap<String, OperatorVersionList>, CmdError> {
    debug!("Building versions list");

    let mut versions_list = IndexMap::new();

    for operator in operator::VALID_OPERATORS {
        for (helm_repo_name, helm_repo_index_file) in helm_index_files {
            let span = tracing::info_span!(
                "build_versions_list_iter",
                helm.repository.name = %helm_repo_name,
                operator_name = %operator,
            );
            let versions =
                span.in_scope(|| list_operator_versions_from_repo(operator, helm_repo_index_file))?;
            let entry = versions_list.entry(operator.to_string());
            let entry = entry.or_insert(OperatorVersionList(HashMap::new()));
            entry.0.insert(helm_repo_name.to_string(), versions);
        }
    }

    Ok(versions_list)
}

/// Builds a list of versions for one operator (by name) which is grouped by
/// stable, test and dev lines based on the list of Helm repo index files.
#[instrument(skip_all, fields(%operator_name))]
fn build_versions_list_for_operator<T>(
    operator_name: T,
    helm_index_files: &HashMap<&str, ChartSourceMetadata>,
) -> Result<OperatorVersionList, CmdError>
where
    T: AsRef<str> + std::fmt::Display + std::fmt::Debug,
{
    debug!("Build versions list for operator");

    let mut versions_list = OperatorVersionList(HashMap::new());
    let operator_name = operator_name.as_ref();

    for (helm_repo_name, helm_repo_index_file) in helm_index_files {
        let versions = list_operator_versions_from_repo(operator_name, helm_repo_index_file)?;
        versions_list.0.insert(helm_repo_name.to_string(), versions);
    }

    Ok(versions_list)
}

/// Builds a list of operator versions based on the provided Helm repo.
#[instrument(skip_all, fields(%operator_name))]
fn list_operator_versions_from_repo<T>(
    operator_name: T,
    helm_repo: &ChartSourceMetadata,
) -> Result<Vec<String>, CmdError>
where
    T: AsRef<str> + std::fmt::Display + std::fmt::Debug,
{
    debug!("Listing operator versions from repository");

    let chart_name = utils::operator_chart_name(operator_name.as_ref());

    match helm_repo.entries.get(&chart_name) {
        Some(entries) => {
            let mut versions = entries
                .iter()
                .map(|entry| {
                    tracing::trace!(helm.chart.name = %chart_name, helm.chart.version = %entry.version, "Found operator chart version");
                    Version::parse(&entry.version).with_context(|_| InvalidHelmChartVersionSnafu {
                        version: entry.version.clone(),
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;
            versions.sort();

            Ok(versions.iter().map(|version| version.to_string()).collect())
        }
        None => Ok(vec![]),
    }
}

mod coffee {
    use std::str::FromStr;

    pub const COFFEE_ASCII_ART: &str = r#"
      ) )
     ( (
   .------.
   |      |]
   \      /
    `----'

  Psst... "coffee" is not an operator, but we get it.
  Stackable runs on coffee too. Have a great day! ☕
"#;

    #[derive(Clone, Debug)]
    pub enum OperatorOrCoffee {
        Operator(super::operator::OperatorSpec),
        Coffee,
    }

    impl FromStr for OperatorOrCoffee {
        type Err = super::operator::SpecParseError;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            match s {
                "coffee" | "coffe" => Ok(OperatorOrCoffee::Coffee), // typos:ignore-line
                _ => s.parse().map(OperatorOrCoffee::Operator),
            }
        }
    }
}
