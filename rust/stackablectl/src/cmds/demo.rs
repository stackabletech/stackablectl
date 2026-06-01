use std::sync::Arc;

use clap::{Args, Subcommand};
use comfy_table::{
    ContentArrangement, Row, Table,
    presets::{NOTHING, UTF8_FULL},
};
use dialoguer::Confirm;
use snafu::{OptionExt as _, ResultExt, Snafu, ensure};
use stackable_cockpit::{
    common::list,
    constants::{DEFAULT_NAMESPACE, DEFAULT_OPERATOR_NAMESPACE},
    platform::{
        demo::{self, DemoInstallParameters, DemoUninstallParameters},
        operator::ChartSourceType,
        release, stack,
    },
    utils::{
        k8s::{self, Client},
        path::PathOrUrlParseError,
    },
    xfer,
};
use stackable_operator::kvp::{LabelError, Labels};
use tracing::{Span, debug, info, instrument};
use tracing_indicatif::{self, span_ext::IndicatifSpanExt as _};

use crate::{
    args::{CommonClusterArgs, CommonClusterArgsError, CommonNamespaceArgs, CommonPromptArgs},
    cli::{Cli, OutputType},
    utils::load_operator_values,
};

#[derive(Debug, Args)]
pub struct DemoArgs {
    #[command(subcommand)]
    subcommand: DemoCommands,

    /// Target a specific Stackable release
    #[arg(long, global = true)]
    release: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum DemoCommands {
    /// List available demos
    #[command(alias("ls"))]
    List(DemoListArgs),

    /// Print out detailed demo information
    #[command(alias("desc"))]
    Describe(DemoDescribeArgs),

    /// Install a specific demo
    #[command(aliases(["i", "in"]))]
    Install(DemoInstallArgs),

    /// Uninstall a specific stack. Caution: This will delete the provided stack namespace,
    /// the operators and provided operator namespace, and all Stackable CRDs
    #[command(aliases(["u", "un"]))]
    Uninstall(DemoUninstallArgs),
}

#[derive(Debug, Args)]
pub struct DemoListArgs {
    #[arg(short, long = "output", value_enum, default_value_t = Default::default())]
    output_type: OutputType,
}

#[derive(Debug, Args)]
pub struct DemoDescribeArgs {
    /// Demo to describe
    #[arg(
        name = "DEMO",
        long_help = "Demo to describe

Use \"stackablectl demo list\" to display a list of available demos.
Use \"stackablectl demo install <DEMO>\" to install a specific demo."
    )]
    demo_name: String,

    #[arg(short, long = "output", value_enum, default_value_t = Default::default())]
    output_type: OutputType,
}

#[derive(Debug, Args)]
pub struct DemoInstallArgs {
    /// Demo to install
    #[arg(
        name = "DEMO",
        long_help = "Demo to install

Use \"stackablectl demo list\" to display a list of available demos.
Use \"stackablectl demo describe <DEMO>\" to display details about the specified demo."
    )]
    demo_name: String,

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

    /// List of parameters to use when installing the demo
    #[arg(long)]
    parameters: Vec<String>,

    #[command(flatten)]
    local_cluster: CommonClusterArgs,

    #[command(flatten)]
    namespaces: CommonNamespaceArgs,

    #[command(flatten)]
    prompt_args: CommonPromptArgs,
}

#[derive(Debug, Args)]
pub struct DemoUninstallArgs {
    /// Demo to uninstall
    demo_name: String,

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
    #[snafu(display("failed to serialize YAML output"))]
    SerializeYamlOutput { source: serde_yaml::Error },

    #[snafu(display("failed to serialize JSON output"))]
    SerializeJsonOutput { source: serde_json::Error },

    #[snafu(display("no demo with name {name:?}"))]
    NoSuchDemo { name: String },

    #[snafu(display("no release {release:?}"))]
    NoSuchRelease { release: String },

    #[snafu(display("failed to get latest release"))]
    LatestRelease,

    #[snafu(display("failed to build demo/stack/release list"))]
    BuildList { source: list::Error },

    #[snafu(display("path/url parse error"))]
    PathOrUrlParse { source: PathOrUrlParseError },

    #[snafu(display("failed to install local cluster"))]
    InstallCluster { source: CommonClusterArgsError },

    #[snafu(display("failed to install demo {demo_name:?}"))]
    InstallDemo {
        source: demo::Error,
        demo_name: String,
    },

    #[snafu(display("failed to confirm user input"))]
    ConfirmDialog { source: dialoguer::Error },

    #[snafu(display("failed to uninstall demo {demo_name:?}"))]
    UninstallDemo {
        source: demo::Error,
        demo_name: String,
    },

    #[snafu(display(
        "demo {demo_name:?} cannot be uninstalled from the {DEFAULT_NAMESPACE:?} namespace, because Kubernetes does not allow deleting it. Pass --namespace <NAMESPACE> if the demo was installed in a different namespace, or delete the demo's resources manually"
    ))]
    UninstallFromDefaultNamespace { demo_name: String },

    #[snafu(display("failed to build labels for demo resources"))]
    BuildLabels { source: LabelError },

    #[snafu(display("failed to create Kubernetes client"))]
    KubeClientCreate { source: k8s::Error },

    #[snafu(display("failed to load operator values"))]
    LoadOperatorValues { source: crate::utils::Error },
}

impl DemoArgs {
    #[instrument(skip_all, fields(with_cache = !cli.no_cache))]
    pub async fn run(
        &self,
        cli: &Cli,
        transfer_client: Arc<xfer::Client>,
    ) -> Result<String, CmdError> {
        debug!("Handle demo args");

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

        // Build demo list based on the (default) remote demo file, and additional files provided by the
        // STACKABLE_DEMO_FILES env variable or the --demo-files CLI argument.
        let demo_files = cli
            .get_demo_files(&release_branch)
            .context(PathOrUrlParseSnafu)?;

        let list = demo::List::build(&demo_files, &transfer_client)
            .await
            .context(BuildListSnafu)?;

        match &self.subcommand {
            DemoCommands::List(args) => list_cmd(args, list).await,
            DemoCommands::Describe(args) => describe_cmd(args, list).await,
            DemoCommands::Install(args) => {
                install_cmd(args, cli, list, &transfer_client, &release_branch).await
            }
            DemoCommands::Uninstall(args) => {
                uninstall_cmd(args, cli, list, &transfer_client, &release_branch).await
            }
        }
    }
}

/// Print out a list of demos, either as a table (plain), JSON or YAML
#[instrument(skip_all, fields(indicatif.pb_show = true))]
async fn list_cmd(args: &DemoListArgs, list: demo::List) -> Result<String, CmdError> {
    info!("Listing demos");
    Span::current().pb_set_message("Fetching demo information");

    match args.output_type {
        OutputType::Plain | OutputType::Table => {
            let (arrangement, preset) = match args.output_type {
                OutputType::Plain => (ContentArrangement::Disabled, NOTHING),
                _ => (ContentArrangement::Dynamic, UTF8_FULL),
            };

            let mut table = Table::new();
            table
                .set_header(vec!["#", "NAME", "STACK", "DESCRIPTION"])
                .set_content_arrangement(arrangement)
                .load_preset(preset);

            for (index, (demo_name, demo_spec)) in list.iter().enumerate() {
                let row = Row::from(vec![
                    (index + 1).to_string(),
                    demo_name.clone(),
                    demo_spec.stack.clone(),
                    demo_spec.description.clone(),
                ]);
                table.add_row(row);
            }

            let mut result = Cli::result();

            result
                .with_command_hint(
                    "stackablectl demo describe [OPTIONS] <DEMO>",
                    "display further information for the specified demo",
                )
                .with_command_hint(
                    "stackablectl demo install [OPTIONS] <DEMO>",
                    "install a demo",
                )
                .with_output(table.to_string());

            Ok(result.render())
        }
        OutputType::Json => serde_json::to_string(&*list).context(SerializeJsonOutputSnafu),
        OutputType::Yaml => serde_yaml::to_string(&*list).context(SerializeYamlOutputSnafu),
    }
}

/// Describe a specific demo by printing out a table (plain), JSON or YAML
#[instrument(skip_all, fields(indicatif.pb_show = true))]
async fn describe_cmd(args: &DemoDescribeArgs, list: demo::List) -> Result<String, CmdError> {
    info!(demo_name = %args.demo_name, "Describing demo");
    Span::current().pb_set_message("Fetching demo information");

    let demo = list.get(&args.demo_name).ok_or(CmdError::NoSuchDemo {
        name: args.demo_name.clone(),
    })?;

    match args.output_type {
        OutputType::Plain | OutputType::Table => {
            let arrangement = match args.output_type {
                OutputType::Plain => ContentArrangement::Disabled,
                _ => ContentArrangement::Dynamic,
            };

            let mut table = Table::new();
            table
                .set_content_arrangement(arrangement)
                .load_preset(NOTHING)
                .add_row(vec!["DEMO", &args.demo_name])
                .add_row(vec!["DESCRIPTION", &demo.description])
                .add_row_if(
                    |_, _| demo.documentation.is_some(),
                    // The simple unwrap() may be called despite the condition, if add_row_if evaluates its arguments eagerly, so make sure to avoid a panic
                    demo.documentation
                        .as_ref()
                        .map(|doc| vec!["DOCUMENTATION", doc])
                        .unwrap_or_else(Vec::new),
                )
                .add_row(vec!["STACK", &demo.stack])
                .add_row(vec!["LABELS", &demo.labels.join(", ")]);

            // TODO (Techassi): Add parameter output

            let mut result = Cli::result();

            result
                .with_command_hint(
                    format!(
                        "stackablectl demo install {demo_name}",
                        demo_name = args.demo_name
                    ),
                    "install the demo",
                )
                .with_command_hint("stackablectl demo list", "list all available demos")
                .with_output(table.to_string());

            Ok(result.render())
        }
        OutputType::Json => serde_json::to_string(&demo).context(SerializeJsonOutputSnafu),
        OutputType::Yaml => serde_yaml::to_string(&demo).context(SerializeYamlOutputSnafu),
    }
}

/// Install a specific demo
#[instrument(skip_all, fields(
    demo_name = %args.demo_name,
    skip_release = args.skip_release,
    %release_branch,
    indicatif.pb_show = true
))]
async fn install_cmd(
    args: &DemoInstallArgs,
    cli: &Cli,
    list: demo::List,
    transfer_client: &xfer::Client,
    release_branch: &str,
) -> Result<String, CmdError> {
    info!(demo_name = %args.demo_name, "Installing demo");
    Span::current().pb_set_message(&format!(
        "Installing demo {demo_name}",
        demo_name = args.demo_name
    ));

    // Init result output and progress output
    let mut output = Cli::result();

    let demo = list.get(&args.demo_name).ok_or(CmdError::NoSuchDemo {
        name: args.demo_name.clone(),
    })?;

    // TODO (Techassi): Try to move all this boilerplate code to build the lists out of here
    let files = cli
        .get_stack_files(release_branch)
        .context(PathOrUrlParseSnafu)?;
    let stack_list = stack::StackList::build(&files, transfer_client)
        .await
        .context(BuildListSnafu)?;

    let files = cli.get_release_files().context(PathOrUrlParseSnafu)?;
    let release_list = release::ReleaseList::build(&files, transfer_client)
        .await
        .context(BuildListSnafu)?;

    // Install local cluster if needed
    args.local_cluster
        .install_if_needed()
        .await
        .context(InstallClusterSnafu)?;

    let client = Client::new().await.context(KubeClientCreateSnafu)?;

    // Construct labels which get attached to all dynamic objects which
    // are part of the demo and stack.
    let labels = Labels::try_from([
        ("stackable.tech/managed-by", "stackablectl"),
        ("stackable.tech/demo", &args.demo_name),
        ("stackable.tech/vendor", "Stackable"),
    ])
    .context(BuildLabelsSnafu)?;

    let mut stack_labels = labels.clone();
    stack_labels
        .parse_insert(("stackable.tech/stack", &demo.stack))
        .context(BuildLabelsSnafu)?;

    // `stackablectl demo uninstall` relies on namespace deletion, suggest installing in a non-default namespace
    // It should still be possible to skip that if either uninstall is not needed
    // or installing an older version of the demo which only supports the 'default' namespace
    let non_default_namespace_confirmation = || -> Result<bool, CmdError> {
        // Ask to install in a non-default namespace, currently suggesting the demo name as namespace name
        Confirm::new()
        .with_prompt(
            format!(
                "Demos installed in the {DEFAULT_NAMESPACE:?} namespace cannot be uninstalled with stackablectl. Install the demo in the {demo_namespace:?} namespace instead?",
            demo_namespace = args.demo_name.clone())
        )
        .default(true)
        .interact()
        .context(ConfirmDialogSnafu)
    };

    let demo_namespace = if args.namespaces.namespace == DEFAULT_NAMESPACE {
        if args.prompt_args.assume_yes
            || tracing_indicatif::suspend_tracing_indicatif(non_default_namespace_confirmation)?
        {
            // User selected to install in suggested namespace
            args.demo_name.clone()
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

    let install_parameters = DemoInstallParameters {
        stack_name: demo.stack.clone(),
        demo_name: args.demo_name.clone(),
        operator_namespace: args.namespaces.operator_namespace.clone(),
        demo_namespace: demo_namespace.clone(),
        stack_parameters: args.stack_parameters.clone(),
        parameters: args.parameters.clone(),
        skip_release: args.skip_release,
        stack_labels,
        labels,
        chart_source: ChartSourceType::from(cli.chart_type()),
        operator_values,
    };

    demo.install(
        stack_list,
        release_list,
        install_parameters,
        &client,
        transfer_client,
    )
    .await
    .context(InstallDemoSnafu {
        demo_name: args.demo_name.clone(),
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
        option = if demo_namespace != DEFAULT_NAMESPACE {
            format!(" --namespace {namespace}", namespace = demo_namespace)
        } else {
            "".into()
        }
    );

    output
        .with_command_hint(operator_cmd, "display the installed operators")
        .with_command_hint(stacklet_cmd, "display the installed stacklets")
        .with_output(format!(
            "Installed demo {demo_name:?}",
            demo_name = args.demo_name
        ));

    Ok(output.render())
}

#[instrument(skip_all, fields(
    demo_name = %args.demo_name,
    %release_branch,
    indicatif.pb_show = true
))]
async fn uninstall_cmd(
    args: &DemoUninstallArgs,
    cli: &Cli,
    list: demo::List,
    transfer_client: &xfer::Client,
    release_branch: &str,
) -> Result<String, CmdError> {
    // Init result output and progress output
    let mut output = Cli::result();

    ensure!(
        args.namespaces.namespace != DEFAULT_NAMESPACE,
        UninstallFromDefaultNamespaceSnafu {
            demo_name: args.demo_name.clone(),
        }
    );

    let proceed_with_uninstall = args.prompt_args.assume_yes
        || {
            tracing_indicatif::suspend_tracing_indicatif(|| -> Result<bool, CmdError> {
                Confirm::new()
            .with_prompt(
                format!(
                    "Uninstalling the demo {demo_name:?} will delete the {demo_namespace:?} namespace. This action cannot be undone. Proceed?",
                demo_name = args.demo_name.clone(),
                demo_namespace = args.namespaces.namespace.clone())
            )
            .default(true)
            .interact()
            .context(ConfirmDialogSnafu)
            })?
        };

    if !proceed_with_uninstall {
        output.with_output(format!(
            "Uninstalling demo {demo_name:?} canceled",
            demo_name = args.demo_name.clone()
        ));

        return Ok(output.render());
    }

    info!(demo_name = %args.demo_name, "Uninstalling demo");
    Span::current().pb_set_message(&format!(
        "Uninstalling demo {demo_name}",
        demo_name = args.demo_name
    ));

    let demo = list.get(&args.demo_name).ok_or(CmdError::NoSuchDemo {
        name: args.demo_name.clone(),
    })?;

    let stack_files = cli
        .get_stack_files(release_branch)
        .context(PathOrUrlParseSnafu)?;
    let stack_list = stack::StackList::build(&stack_files, transfer_client)
        .await
        .context(BuildListSnafu)?;

    let client = Client::new().await.context(KubeClientCreateSnafu)?;

    let release_files = cli.get_release_files().context(PathOrUrlParseSnafu)?;
    let release_list = release::ReleaseList::build(&release_files, transfer_client)
        .await
        .context(BuildListSnafu)?;

    demo.uninstall(
        stack_list,
        release_list,
        DemoUninstallParameters {
            demo_name: args.demo_name.clone(),
            operator_namespace: args.namespaces.operator_namespace.clone(),
            demo_namespace: args.namespaces.namespace.clone(),
            skip_operators: args.skip_operators_and_crds,
            skip_crds: args.skip_operators_and_crds,
        },
        &client,
        transfer_client,
    )
    .await
    .context(UninstallDemoSnafu {
        demo_name: args.demo_name.clone(),
    })?;

    output.with_output(format!(
        "Uninstalled demo {demo_name:?}",
        demo_name = args.demo_name
    ));

    Ok(output.render())
}
