use std::sync::Arc;

use clap::{Args, Subcommand};
use comfy_table::{
    ContentArrangement, Table,
    presets::{NOTHING, UTF8_FULL},
};
use dialoguer::Confirm;
use snafu::{OptionExt as _, ResultExt, Snafu, ensure};
use stackable_cockpit::{
    common::list,
    constants::{DEFAULT_NAMESPACE, DEFAULT_OPERATOR_NAMESPACE},
    platform::{
        operator::ChartSourceType,
        release,
        stack::{self, StackInstallParameters, StackUninstallParameters},
    },
    utils::{
        k8s::{self, Client},
        path::PathOrUrlParseError,
    },
    xfer,
};
use stackable_operator::kvp::{LabelError, Labels};
use tracing::{Span, debug, info, instrument};
use tracing_indicatif::span_ext::IndicatifSpanExt as _;

use crate::{
    args::{CommonClusterArgs, CommonClusterArgsError, CommonNamespaceArgs, CommonPromptArgs},
    cli::{Cli, OutputType},
    utils::load_operator_values,
};

#[derive(Debug, Args)]
pub struct StackArgs {
    #[command(subcommand)]
    subcommand: StackCommands,

    /// Target a specific Stackable release
    #[arg(long, global = true)]
    release: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum StackCommands {
    /// List available stacks
    #[command(alias("ls"))]
    List(StackListArgs),

    /// Describe a specific stack
    #[command(alias("desc"))]
    Describe(StackDescribeArgs),

    /// Install a specific stack
    #[command(aliases(["i", "in"]))]
    Install(StackInstallArgs),

    /// Uninstall a specific stack. Caution: This will delete the provided stack namespace,
    /// the operators and provided operator namespace, and all Stackable CRDs
    #[command(aliases(["u", "un"]))]
    Uninstall(StackUninstallArgs),
}

#[derive(Debug, Args)]
pub struct StackListArgs {
    #[arg(short, long = "output", value_enum, default_value_t = Default::default())]
    output_type: OutputType,
}

#[derive(Debug, Args)]
pub struct StackDescribeArgs {
    /// Name of the stack to describe
    stack_name: String,

    #[arg(short, long = "output", value_enum, default_value_t = Default::default())]
    output_type: OutputType,
}

#[derive(Debug, Args)]
pub struct StackInstallArgs {
    /// Name of the stack to install
    stack_name: String,

    /// Skip the installation of the release during the stack install process
    #[arg(
        long,
        long_help = "Skip the installation of the release during the stack install process

Use \"stackablectl operator install [OPTIONS] <OPERATORS>...\" to install
required operators manually. Operators MUST be installed in the correct version.

Use \"stackablectl operator install --help\" to display more information on how
to specify operator versions."
    )]
    skip_release: bool,

    /// List of parameters to use when installing the stack
    #[arg(long)]
    stack_parameters: Vec<String>,

    /// List of parameters to use when installing the stack
    #[arg(long)]
    #[arg(long_help = "List of parameters to use when installing the stack

All parameters must have the format '<parameter>=<value>'. Multiple parameters
can be specified and are space separated. Valid parameters are:

- adminPassword=admin123
- adminUser=superuser
- 'endpoint=https://example.com port=1234'

Use \"stackablectl stack describe <STACK>\" to list available parameters for each stack.")]
    parameters: Vec<String>,

    #[command(flatten)]
    local_cluster: CommonClusterArgs,

    #[command(flatten)]
    namespaces: CommonNamespaceArgs,

    #[command(flatten)]
    prompt_args: CommonPromptArgs,
}

#[derive(Debug, Args)]
pub struct StackUninstallArgs {
    /// Name of the stack to uninstall
    stack_name: String,

    #[command(flatten)]
    namespaces: CommonNamespaceArgs,

    /// Skip uninstalling Stackable operators and CRDs
    #[arg(long)]
    skip_operators_and_crds: bool,

    #[command(flatten)]
    prompt_args: CommonPromptArgs,
}

#[derive(Debug, Snafu)]
pub enum CmdError {
    #[snafu(display("path/url parse error"))]
    PathOrUrlParse { source: PathOrUrlParseError },

    #[snafu(display("failed to serialize YAML output"))]
    SerializeYamlOutput { source: serde_yaml::Error },

    #[snafu(display("failed to serialize JSON output"))]
    SerializeJsonOutput { source: serde_json::Error },

    #[snafu(display("no release {release:?}"))]
    NoSuchRelease { release: String },

    #[snafu(display("failed to get latest release"))]
    LatestRelease,

    #[snafu(display("failed to build stack/release list"))]
    BuildList { source: list::Error },

    #[snafu(display("failed to install local cluster"))]
    InstallCluster { source: CommonClusterArgsError },

    #[snafu(display("failed to install stack {stack_name:?}"))]
    InstallStack {
        #[snafu(source(from(stack::Error, Box::new)))]
        source: Box<stack::Error>,
        stack_name: String,
    },

    #[snafu(display("failed to confirm user input"))]
    ConfirmDialog { source: dialoguer::Error },

    #[snafu(display("failed to uninstall stack {stack_name:?}"))]
    UninstallStack {
        #[snafu(source(from(stack::Error, Box::new)))]
        source: Box<stack::Error>,
        stack_name: String,
    },

    #[snafu(display(
        "stack {stack_name:?} cannot be uninstalled from the {DEFAULT_NAMESPACE:?} namespace, because Kubernetes does not allow deleting it. Pass --namespace <NAMESPACE> if the stack was installed in a different namespace, or delete the stack's resources manually"
    ))]
    UninstallFromDefaultNamespace { stack_name: String },

    #[snafu(display("failed to build labels for stack resources"))]
    BuildLabels { source: LabelError },

    #[snafu(display("failed to create Kubernetes client"))]
    KubeClientCreate {
        #[snafu(source(from(k8s::Error, Box::new)))]
        source: Box<k8s::Error>,
    },

    #[snafu(display("failed to load operator values"))]
    LoadOperatorValues { source: crate::utils::Error },
}

impl StackArgs {
    pub async fn run(
        &self,
        cli: &Cli,
        transfer_client: Arc<xfer::Client>,
    ) -> Result<String, CmdError> {
        debug!("Handle stack args");

        let release_files = cli.get_release_files().context(PathOrUrlParseSnafu)?;
        let release_list = release::ReleaseList::build(&release_files, &transfer_client)
            .await
            .context(BuildListSnafu)?;

        let release_branch = match &self.release {
            Some(release) => {
                ensure!(
                    release_list.contains_key(release),
                    NoSuchReleaseSnafu { release }
                );

                if release == "dev" {
                    "main".to_string()
                } else {
                    format!("release-{release}")
                }
            }
            None => {
                let (release_name, _) = release_list.first().context(LatestReleaseSnafu)?;
                format!("release-{release_name}")
            }
        };

        let files = cli
            .get_stack_files(&release_branch)
            .context(PathOrUrlParseSnafu)?;
        let stack_list = stack::StackList::build(&files, &transfer_client)
            .await
            .context(BuildListSnafu)?;

        match &self.subcommand {
            StackCommands::List(args) => list_cmd(args, stack_list),
            StackCommands::Describe(args) => describe_cmd(args, stack_list),
            StackCommands::Install(args) => {
                install_cmd(args, cli, stack_list, &transfer_client).await
            }
            StackCommands::Uninstall(args) => {
                uninstall_cmd(args, cli, stack_list, &transfer_client).await
            }
        }
    }
}

#[instrument(skip_all, fields(indicatif.pb_show = true))]
fn list_cmd(args: &StackListArgs, stack_list: stack::StackList) -> Result<String, CmdError> {
    info!("Listing stacks");
    Span::current().pb_set_message("Fetching stack information");

    match args.output_type {
        OutputType::Plain | OutputType::Table => {
            let (arrangement, preset) = match args.output_type {
                OutputType::Plain => (ContentArrangement::Disabled, NOTHING),
                _ => (ContentArrangement::Dynamic, UTF8_FULL),
            };

            let mut table = Table::new();
            table
                .set_header(vec!["#", "STACK", "RELEASE", "DESCRIPTION"])
                .set_content_arrangement(arrangement)
                .load_preset(preset);

            for (index, (stack_name, stack)) in stack_list.iter().enumerate() {
                table.add_row(vec![
                    (index + 1).to_string(),
                    stack_name.clone(),
                    stack.release.clone(),
                    stack.description.clone(),
                ]);
            }

            let mut result = Cli::result();

            result
                .with_command_hint(
                    "stackablectl stack describe [OPTIONS] <STACK>",
                    "display further information for the specified stack",
                )
                .with_command_hint(
                    "stackablectl stack install [OPTIONS] <STACK>...",
                    "install a stack",
                )
                .with_output(table.to_string());

            Ok(result.render())
        }
        OutputType::Json => serde_json::to_string(&stack_list).context(SerializeJsonOutputSnafu),
        OutputType::Yaml => serde_yaml::to_string(&stack_list).context(SerializeYamlOutputSnafu),
    }
}

#[instrument(skip_all, fields(indicatif.pb_show = true))]
fn describe_cmd(
    args: &StackDescribeArgs,
    stack_list: stack::StackList,
) -> Result<String, CmdError> {
    info!(stack_name = %args.stack_name, "Describing stack");
    Span::current().pb_set_message("Fetching stack information");

    match stack_list.get(&args.stack_name) {
        Some(stack) => match args.output_type {
            OutputType::Plain | OutputType::Table => {
                let arrangement = match args.output_type {
                    OutputType::Plain => ContentArrangement::Disabled,
                    _ => ContentArrangement::Dynamic,
                };

                let mut table = Table::new();

                let mut parameter_table = Table::new();

                parameter_table
                    .set_header(vec!["NAME", "DESCRIPTION", "DEFAULT VALUE"])
                    .set_content_arrangement(arrangement.clone())
                    .load_preset(NOTHING);

                for parameter in &stack.parameters {
                    parameter_table.add_row(vec![
                        parameter.name.clone(),
                        parameter.description.clone(),
                        parameter.default.clone(),
                    ]);
                }

                table
                    .set_content_arrangement(arrangement)
                    .load_preset(NOTHING)
                    .add_row(vec!["STACK", args.stack_name.as_str()])
                    .add_row(vec!["DESCRIPTION", stack.description.as_str()])
                    .add_row(vec!["RELEASE", stack.release.as_str()])
                    .add_row(vec!["OPERATORS", stack.operators.join(", ").as_str()])
                    .add_row(vec!["LABELS", stack.labels.join(", ").as_str()])
                    .add_row(vec!["PARAMETERS", parameter_table.to_string().as_str()]);

                let mut result = Cli::result();

                result
                    .with_command_hint(
                        format!(
                            "stackablectl stack install {stack_name}",
                            stack_name = args.stack_name
                        ),
                        "install the stack",
                    )
                    .with_command_hint("stackablectl stack list", "list all available stacks")
                    .with_output(table.to_string());

                Ok(result.render())
            }
            OutputType::Json => serde_json::to_string(&stack).context(SerializeJsonOutputSnafu),
            OutputType::Yaml => serde_yaml::to_string(&stack).context(SerializeYamlOutputSnafu),
        },
        None => Ok("No such stack".into()),
    }
}

#[instrument(skip(cli, stack_list, transfer_client), fields(indicatif.pb_show = true))]
async fn install_cmd(
    args: &StackInstallArgs,
    cli: &Cli,
    stack_list: stack::StackList,
    transfer_client: &xfer::Client,
) -> Result<String, CmdError> {
    info!(stack_name = %args.stack_name, "Installing stack");
    Span::current().pb_set_message(&format!(
        "Installing stack {stack_name}",
        stack_name = args.stack_name
    ));

    let files = cli.get_release_files().context(PathOrUrlParseSnafu)?;
    let release_list = release::ReleaseList::build(&files, transfer_client)
        .await
        .context(BuildListSnafu)?;

    match stack_list.get(&args.stack_name) {
        Some(stack_spec) => {
            let mut output = Cli::result();

            // Install local cluster if needed
            args.local_cluster
                .install_if_needed()
                .await
                .context(InstallClusterSnafu)?;

            let client = Client::new().await.context(KubeClientCreateSnafu)?;

            // Construct labels which get attached to all dynamic objects which
            // are part of the stack.
            let labels = Labels::try_from([
                ("stackable.tech/managed-by", "stackablectl"),
                ("stackable.tech/stack", &args.stack_name),
                ("stackable.tech/vendor", "Stackable"),
            ])
            .context(BuildLabelsSnafu)?;

            // `stackablectl stack uninstall` relies on namespace deletion, suggest installing in a non-default namespace
            // It should still be possible to skip that if either uninstall is not needed
            // or installing an older version of the stack which only supports the 'default' namespace
            let non_default_namespace_confirmation = || -> Result<bool, CmdError> {
                // Ask to install in a non-default namespace, currently suggesting the stack name as namespace name
                Confirm::new()
                .with_prompt(
                    format!(
                        "Stacks installed in the {DEFAULT_NAMESPACE:?} namespace cannot be uninstalled with stackablectl. Install the stack in the {stack_namespace:?} namespace instead?",
                    stack_namespace = args.stack_name.clone())
                )
                .default(true)
                .interact()
                .context(ConfirmDialogSnafu)
            };

            let stack_namespace = if args.namespaces.namespace == DEFAULT_NAMESPACE {
                if args.prompt_args.assume_yes
                    || tracing_indicatif::suspend_tracing_indicatif(
                        non_default_namespace_confirmation,
                    )?
                {
                    // User selected to install in suggested namespace
                    args.stack_name.clone()
                } else {
                    // User selected to install in default namespace
                    args.namespaces.namespace.clone()
                }
            } else {
                // User provided a non-default namespace with command argument
                args.namespaces.namespace.clone()
            };

            let values_file = cli.get_values_file().context(PathOrUrlParseSnafu)?;
            let operator_values = load_operator_values(values_file.as_ref(), transfer_client)
                .await
                .context(LoadOperatorValuesSnafu)?;

            let install_parameters = StackInstallParameters {
                stack_name: args.stack_name.clone(),
                // There is no demo when installing only a stack
                demo_name: None,
                operator_namespace: args.namespaces.operator_namespace.clone(),
                stack_namespace: stack_namespace.clone(),
                parameters: args.parameters.clone(),
                skip_release: args.skip_release,
                labels,
                chart_source: ChartSourceType::from(cli.chart_type()),
                operator_values,
            };

            stack_spec
                .install(release_list, install_parameters, &client, transfer_client)
                .await
                .context(InstallStackSnafu {
                    stack_name: args.stack_name.clone(),
                })?;

            let operator_cmd = format!(
                "stackablectl operator installed{option}",
                option = if args.namespaces.operator_namespace != DEFAULT_OPERATOR_NAMESPACE {
                    format!(
                        " --operator-namespace {namespace}",
                        namespace = args.namespaces.operator_namespace
                    )
                } else {
                    "".into()
                }
            );

            let stacklet_cmd = format!(
                "stackablectl stacklet list{option}",
                option = if stack_namespace != DEFAULT_NAMESPACE {
                    format!(" --namespace {namespace}", namespace = stack_namespace)
                } else {
                    "".into()
                }
            );

            output
                .with_command_hint(operator_cmd, "display the installed operators")
                .with_command_hint(stacklet_cmd, "display the installed stacklets")
                .with_output(format!(
                    "Installed stack {stack_name:?}",
                    stack_name = args.stack_name
                ));

            Ok(output.render())
        }
        None => Ok("No such stack".into()),
    }
}

#[instrument(skip(cli, stack_list, transfer_client), fields(indicatif.pb_show = true))]
async fn uninstall_cmd(
    args: &StackUninstallArgs,
    cli: &Cli,
    stack_list: stack::StackList,
    transfer_client: &xfer::Client,
) -> Result<String, CmdError> {
    let mut output = Cli::result();

    ensure!(
        args.namespaces.namespace != DEFAULT_NAMESPACE,
        UninstallFromDefaultNamespaceSnafu {
            stack_name: args.stack_name.clone(),
        }
    );

    let proceed_with_uninstall = args.prompt_args.assume_yes
        || tracing_indicatif::suspend_tracing_indicatif(|| -> Result<bool, CmdError> {
            Confirm::new()
            .with_prompt(
                format!(
                    "Uninstalling the stack {stack_name:?} will delete the {stack_namespace:?} namespace. This action cannot be undone. Proceed?",
                stack_name = args.stack_name.clone(),
                stack_namespace = args.namespaces.namespace.clone())
            )
            .default(true)
            .interact()
            .context(ConfirmDialogSnafu)
        })?;

    if !proceed_with_uninstall {
        output.with_output(format!(
            "Uninstalling stack {stack_name:?} canceled",
            stack_name = args.stack_name.clone()
        ));

        return Ok(output.render());
    }

    info!(stack_name = %args.stack_name, "Uninstalling stack");
    Span::current().pb_set_message(&format!(
        "Uninstalling stack {stack_name}",
        stack_name = args.stack_name
    ));

    match stack_list.get(&args.stack_name) {
        Some(stack) => {
            let client = Client::new().await.context(KubeClientCreateSnafu)?;

            let files = cli.get_release_files().context(PathOrUrlParseSnafu)?;
            let release_list = release::ReleaseList::build(&files, transfer_client)
                .await
                .context(BuildListSnafu)?;

            stack
                .uninstall(
                    release_list,
                    StackUninstallParameters {
                        stack_name: args.stack_name.clone(),
                        // There is no demo when uninstalling only a stack
                        demo_name: None,
                        operator_namespace: args.namespaces.operator_namespace.clone(),
                        stack_namespace: args.namespaces.namespace.clone(),
                        skip_operators: args.skip_operators_and_crds,
                        skip_crds: args.skip_operators_and_crds,
                    },
                    &client,
                    transfer_client,
                )
                .await
                .context(UninstallStackSnafu {
                    stack_name: args.stack_name.clone(),
                })?;

            output.with_output(format!(
                "Uninstalled stack {stack_name:?}",
                stack_name = args.stack_name
            ));

            Ok(output.render())
        }
        None => Ok("No such stack".into()),
    }
}
