use std::sync::Arc;

use clap::{Args, Subcommand};
use comfy_table::{
    ContentArrangement, Table,
    presets::{NOTHING, UTF8_FULL},
};
use snafu::{ResultExt, Snafu};
use stackable_cockpit::{
    common::list,
    constants::DEFAULT_OPERATOR_NAMESPACE,
    helm::{self, Release},
    platform::{
        namespace,
        operator::{self, ChartSourceType},
        release,
    },
    utils::{
        self,
        k8s::{self, Client},
        path::PathOrUrlParseError,
    },
    xfer,
};
use tracing::{Span, debug, info, instrument};
use tracing_indicatif::span_ext::IndicatifSpanExt as _;

use crate::{
    args::{CommonClusterArgs, CommonClusterArgsError},
    cli::{Cli, OutputType},
    utils::load_operator_values,
};

#[derive(Debug, Args)]
pub struct ReleaseArgs {
    #[command(subcommand)]
    subcommand: ReleaseCommands,
}

#[derive(Debug, Subcommand)]
pub enum ReleaseCommands {
    /// List available releases
    #[command(alias("ls"))]
    List(ReleaseListArgs),

    /// Print out detailed release information
    #[command(alias("desc"))]
    Describe(ReleaseDescribeArgs),

    /// Install a specific release
    #[command(aliases(["i", "in"]))]
    Install(ReleaseInstallArgs),

    /// Uninstall a release
    #[command(aliases(["rm", "un"]))]
    Uninstall(ReleaseUninstallArgs),

    /// Upgrade a release
    Upgrade(ReleaseUpgradeArgs),
}

#[derive(Debug, Args)]
pub struct ReleaseListArgs {
    #[arg(short, long = "output", value_enum, default_value_t = Default::default())]
    output_type: OutputType,
}

#[derive(Debug, Args)]
pub struct ReleaseDescribeArgs {
    #[arg(name = "RELEASE")]
    release: String,

    #[arg(short, long = "output", value_enum, default_value_t = Default::default())]
    output_type: OutputType,
}

#[derive(Debug, Args)]
pub struct ReleaseInstallArgs {
    /// Release to install
    #[arg(name = "RELEASE")]
    release: String,

    /// Whitelist of product operators to install
    #[arg(short, long = "include", group = "products")]
    included_products: Vec<String>,

    /// Blacklist of product operators to install
    #[arg(short, long = "exclude", group = "products")]
    excluded_products: Vec<String>,

    /// Namespace in the cluster used to deploy the operators
    #[arg(long, default_value = DEFAULT_OPERATOR_NAMESPACE, visible_aliases(["operator-ns"]))]
    pub operator_namespace: String,

    #[command(flatten)]
    local_cluster: CommonClusterArgs,
}

#[derive(Debug, Args)]
pub struct ReleaseUpgradeArgs {
    /// Upgrade to the specified release
    #[arg(name = "RELEASE")]
    release: String,

    /// List of product operators to upgrade
    #[arg(short, long = "include", group = "products")]
    included_products: Vec<String>,

    /// Blacklist of product operators to install
    #[arg(short, long = "exclude", group = "products")]
    excluded_products: Vec<String>,

    /// Namespace in the cluster used to deploy the operators
    #[arg(long, default_value = DEFAULT_OPERATOR_NAMESPACE, visible_aliases(["operator-ns"]))]
    pub operator_namespace: String,
}

#[derive(Debug, Args)]
pub struct ReleaseUninstallArgs {
    /// Name of the release to uninstall
    #[arg(name = "RELEASE")]
    release: String,

    /// Namespace in the cluster used to deploy the operators
    #[arg(long, default_value = DEFAULT_OPERATOR_NAMESPACE, visible_aliases(["operator-ns"]))]
    pub operator_namespace: String,
}

#[derive(Debug, Snafu)]
pub enum CmdError {
    #[snafu(display("Helm error"))]
    HelmError { source: helm::Error },

    #[snafu(display("failed to serialize YAML output"))]
    SerializeYamlOutput { source: yaml_serde::Error },

    #[snafu(display("failed to serialize JSON output"))]
    SerializeJsonOutput { source: serde_json::Error },

    #[snafu(display("failed to parse path/url"))]
    PathOrUrlParse { source: PathOrUrlParseError },

    #[snafu(display("failed to build release list"))]
    BuildList { source: list::Error },

    #[snafu(display("no release {release:?}"))]
    NoSuchRelease { release: String },

    #[snafu(display("failed to install release"))]
    ReleaseInstall { source: release::Error },

    #[snafu(display("failed to upgrade CRDs for release"))]
    CrdUpgrade { source: release::Error },

    #[snafu(display("failed to uninstall release"))]
    ReleaseUninstall { source: release::Error },

    #[snafu(display("cluster argument error"))]
    CommonClusterArgs { source: CommonClusterArgsError },

    #[snafu(display("failed to create Kubernetes client"))]
    KubeClientCreate { source: k8s::Error },

    #[snafu(display("failed to create namespace {namespace:?}"))]
    NamespaceCreate {
        source: namespace::Error,
        namespace: String,
    },

    #[snafu(display("failed to load operator values"))]
    LoadOperatorValues { source: crate::utils::Error },
}

impl ReleaseArgs {
    pub async fn run(
        &self,
        cli: &Cli,
        transfer_client: Arc<xfer::Client>,
    ) -> Result<String, CmdError> {
        debug!("Handle release args");

        let files = cli.get_release_files().context(PathOrUrlParseSnafu)?;
        let release_list = release::ReleaseList::build(&files, &transfer_client)
            .await
            .context(BuildListSnafu)?;

        if release_list.is_empty() {
            return Ok("No releases".into());
        }

        match &self.subcommand {
            ReleaseCommands::List(args) => list_cmd(args, release_list).await,
            ReleaseCommands::Describe(args) => describe_cmd(args, release_list).await,
            ReleaseCommands::Install(args) => {
                install_cmd(args, cli, release_list, &transfer_client).await
            }
            ReleaseCommands::Uninstall(args) => uninstall_cmd(args, release_list).await,
            ReleaseCommands::Upgrade(args) => {
                upgrade_cmd(args, cli, release_list, &transfer_client).await
            }
        }
    }
}

#[instrument(skip(release_list), fields(indicatif.pb_show = true))]
async fn list_cmd(
    args: &ReleaseListArgs,
    release_list: release::ReleaseList,
) -> Result<String, CmdError> {
    info!("Listing releases");
    Span::current().pb_set_message("Fetching release information");

    match args.output_type {
        OutputType::Plain | OutputType::Table => {
            if release_list.is_empty() {
                return Ok("No releases".into());
            }

            let (arrangement, preset) = match args.output_type {
                OutputType::Plain => (ContentArrangement::Disabled, NOTHING),
                _ => (ContentArrangement::Dynamic, UTF8_FULL),
            };

            let mut table = Table::new();
            table
                .set_header(vec!["#", "RELEASE", "RELEASE DATE", "DESCRIPTION"])
                .set_content_arrangement(arrangement)
                .load_preset(preset);

            for (index, (release_name, release_spec)) in release_list.iter().enumerate() {
                table.add_row(vec![
                    (index + 1).to_string(),
                    release_name.to_string(),
                    release_spec.date.clone(),
                    release_spec.description.clone(),
                ]);
            }

            let mut result = Cli::result();

            result
                .with_command_hint(
                    "stackablectl release describe [OPTIONS] <RELEASE>",
                    "display further information for the specified release",
                )
                .with_command_hint(
                    "stackablectl release install [OPTIONS] <RELEASE>",
                    "install a release",
                )
                .with_output(table.to_string());

            Ok(result.render())
        }
        OutputType::Json => serde_json::to_string(&release_list).context(SerializeJsonOutputSnafu),
        OutputType::Yaml => yaml_serde::to_string(&release_list).context(SerializeYamlOutputSnafu),
    }
}

#[instrument(skip(release_list), fields(indicatif.pb_show = true))]
async fn describe_cmd(
    args: &ReleaseDescribeArgs,
    release_list: release::ReleaseList,
) -> Result<String, CmdError> {
    info!(release = %args.release, "Describing release");
    Span::current().pb_set_message("Fetching release information");

    let release = release_list.get(&args.release);

    match release {
        Some(release) => match args.output_type {
            OutputType::Plain | OutputType::Table => {
                let arrangement = match args.output_type {
                    OutputType::Plain => ContentArrangement::Disabled,
                    _ => ContentArrangement::Dynamic,
                };

                let mut product_table = Table::new();

                product_table
                    .set_header(vec!["PRODUCT", "OPERATOR VERSION"])
                    .set_content_arrangement(ContentArrangement::Dynamic)
                    .load_preset(NOTHING);

                for (product_name, product) in &release.products {
                    product_table.add_row(vec![product_name, &product.version.to_string()]);
                }

                let mut table = Table::new();

                table
                    .set_content_arrangement(arrangement)
                    .load_preset(NOTHING)
                    .add_row(vec!["RELEASE", &args.release])
                    .add_row(vec!["RELEASE DATE", release.date.as_str()])
                    .add_row(vec!["DESCRIPTION", release.description.as_str()])
                    .add_row(vec![
                        "INCLUDED PRODUCTS",
                        product_table.to_string().as_str(),
                    ]);

                let mut result = Cli::result();

                result
                    .with_command_hint(
                        format!(
                            "stackablectl release install {release}",
                            release = args.release
                        ),
                        "install the release",
                    )
                    .with_command_hint("stackablectl release list", "list all available releases")
                    .with_output(table.to_string());

                Ok(result.render())
            }
            OutputType::Json => serde_json::to_string(&release).context(SerializeJsonOutputSnafu),
            OutputType::Yaml => yaml_serde::to_string(&release).context(SerializeYamlOutputSnafu),
        },
        None => Err(CmdError::NoSuchRelease {
            release: args.release.clone(),
        }),
    }
}

#[instrument(skip(cli, release_list, transfer_client), fields(indicatif.pb_show = true))]
async fn install_cmd(
    args: &ReleaseInstallArgs,
    cli: &Cli,
    release_list: release::ReleaseList,
    transfer_client: &xfer::Client,
) -> Result<String, CmdError> {
    info!(release = %args.release, "Installing release");
    Span::current().pb_set_message("Installing release");

    match release_list.get(&args.release) {
        Some(release) => {
            let mut output = Cli::result();

            // Install local cluster if needed
            args.local_cluster
                .install_if_needed()
                .await
                .context(CommonClusterArgsSnafu)?;

            let client = Client::new().await.context(KubeClientCreateSnafu)?;

            // Create operator namespace if needed
            namespace::create_if_needed(&client, args.operator_namespace.clone())
                .await
                .context(NamespaceCreateSnafu {
                    namespace: args.operator_namespace.clone(),
                })?;

            let values_file = cli.get_values_file().context(PathOrUrlParseSnafu)?;
            let operator_values = load_operator_values(values_file.as_ref(), transfer_client)
                .await
                .context(LoadOperatorValuesSnafu)?;

            release
                .install(
                    &args.included_products,
                    &args.excluded_products,
                    &args.operator_namespace,
                    &ChartSourceType::from(cli.chart_type()),
                    &operator_values,
                )
                .await
                .context(ReleaseInstallSnafu)?;

            output
                .with_command_hint(
                    "stackablectl operator installed",
                    "list installed operators",
                )
                .with_output(format!(
                    "Installed release {release:?}",
                    release = args.release
                ));

            Ok(output.render())
        }
        None => Err(CmdError::NoSuchRelease {
            release: args.release.clone(),
        }),
    }
}

#[instrument(skip_all, fields(indicatif.pb_show = true))]
async fn upgrade_cmd(
    args: &ReleaseUpgradeArgs,
    cli: &Cli,
    release_list: release::ReleaseList,
    transfer_client: &xfer::Client,
) -> Result<String, CmdError> {
    info!(release = %args.release, "Upgrading release");
    Span::current().pb_set_message("Upgrading release");

    match release_list.get(&args.release) {
        Some(release) => {
            let mut output = Cli::result();
            let client = Client::new().await.context(KubeClientCreateSnafu)?;

            // Get all currently installed operators to only upgrade those
            let installed_charts: Vec<Release> =
                helm::list_releases(&args.operator_namespace).context(HelmSnafu)?;

            let mut operators: Vec<String> = operator::VALID_OPERATORS
                .iter()
                .filter(|operator| {
                    installed_charts
                        .iter()
                        .any(|release| release.name == utils::operator_chart_name(operator))
                })
                .map(|operator| operator.to_string())
                .collect();

            // Uninstall the old operator release first
            release
                .uninstall(
                    &operators,
                    &args.excluded_products,
                    &args.operator_namespace,
                )
                .context(ReleaseUninstallSnafu)?;

            // If operators were added to args.included_products, install them as well
            for product in &args.included_products {
                if !operators.contains(product) {
                    operators.push(product.clone());
                }
            }

            // Upgrade the CRDs for all the operators to be upgraded
            release
                .upgrade_crds(
                    &operators,
                    &args.excluded_products,
                    &args.operator_namespace,
                    &client,
                    transfer_client,
                )
                .await
                .context(CrdUpgradeSnafu)?;

            let values_file = cli.get_values_file().context(PathOrUrlParseSnafu)?;
            let operator_values = load_operator_values(values_file.as_ref(), transfer_client)
                .await
                .context(LoadOperatorValuesSnafu)?;

            // Install the new operator release
            release
                .install(
                    &operators,
                    &args.excluded_products,
                    &args.operator_namespace,
                    &ChartSourceType::from(cli.chart_type()),
                    &operator_values,
                )
                .await
                .context(ReleaseInstallSnafu)?;

            output
                .with_command_hint(
                    "stackablectl operator installed",
                    "list installed operators",
                )
                .with_output(format!(
                    "Upgraded to release {release:?}",
                    release = args.release
                ));

            Ok(output.render())
        }
        None => Err(CmdError::NoSuchRelease {
            release: args.release.clone(),
        }),
    }
}

#[instrument(skip(release_list), fields(indicatif.pb_show = true))]
async fn uninstall_cmd(
    args: &ReleaseUninstallArgs,
    release_list: release::ReleaseList,
) -> Result<String, CmdError> {
    Span::current().pb_set_message("Uninstalling release");

    match release_list.get(&args.release) {
        Some(release) => {
            release
                .uninstall(&Vec::new(), &Vec::new(), &args.operator_namespace)
                .context(ReleaseUninstallSnafu)?;

            let mut result = Cli::result();

            result
                .with_command_hint("stackablectl release list", "list available releases")
                .with_output(format!(
                    "Uninstalled release {release:?}",
                    release = args.release
                ));

            Ok(result.render())
        }
        None => Err(CmdError::NoSuchRelease {
            release: args.release.clone(),
        }),
    }
}
