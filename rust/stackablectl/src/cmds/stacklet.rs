use clap::{Args, Subcommand};
use comfy_table::{
    ContentArrangement, Table,
    presets::{NOTHING, UTF8_FULL},
};
use snafu::{ResultExt, Snafu};
use stackable_cockpit::{
    constants::DEFAULT_NAMESPACE,
    platform::stacklet::{self, get_credentials_for_product, list_stacklets},
    utils::k8s::{self, Client, DisplayCondition},
};
use tracing::{Span, info, instrument};
use tracing_indicatif::span_ext::IndicatifSpanExt as _;

use crate::{
    args::CommonNamespaceArgs,
    cli::{Cli, OutputType},
};

#[derive(Debug, Args)]
pub struct StackletArgs {
    #[command(subcommand)]
    subcommand: StackletCommands,
}

#[derive(Debug, Subcommand)]
pub enum StackletCommands {
    /// Display credentials for a stacklet
    #[command(aliases(["creds", "cr"]))]
    Credentials(StackletCredentialsArgs),

    /// List deployed stacklets
    #[command(alias("ls"))]
    List(StackletListArgs),
}

#[derive(Debug, Args)]
pub struct StackletCredentialsArgs {
    /// The name of the product, for example 'superset'.
    product_name: String,

    /// The name of the stacklet, for example 'superset'.
    stacklet_name: String,

    /// Namespace in the cluster used to deploy the products.
    #[arg(
        long,
        short = 'n',
        global = true,
        default_value = DEFAULT_NAMESPACE,
        aliases(["product-ns", "product-namespace"]),
        long_help = "Namespace in the cluster used to deploy the products. Use this to select
a different namespace for credential lookup.")]
    pub namespace: String,
}

#[derive(Debug, Args)]
pub struct StackletListArgs {
    #[arg(short, long = "output", value_enum, default_value_t)]
    output_type: OutputType,

    #[command(flatten)]
    namespaces: CommonNamespaceArgs,
}

#[derive(Debug, Snafu)]
pub enum CmdError {
    #[snafu(display("failed to list stacklets"))]
    StackletList { source: stacklet::Error },

    #[snafu(display("failed to retrieve credentials for stacklet"))]
    StackletCredentials { source: stacklet::Error },

    #[snafu(display("failed to serialize YAML output"))]
    SerializeYamlOutput { source: yaml_serde::Error },

    #[snafu(display("failed to serialize JSON output"))]
    SerializeJsonOutput { source: serde_json::Error },

    #[snafu(display("failed to create Kubernetes client"))]
    KubeClientCreate { source: k8s::Error },
}

impl StackletArgs {
    pub async fn run(&self) -> Result<String, CmdError> {
        match &self.subcommand {
            StackletCommands::List(args) => list_cmd(args).await,
            StackletCommands::Credentials(args) => credentials_cmd(args).await,
        }
    }
}

#[instrument(skip_all, fields(indicatif.pb_show = true))]
async fn list_cmd(args: &StackletListArgs) -> Result<String, CmdError> {
    info!("Listing installed stacklets");
    Span::current().pb_set_message("Fetching stacklet information");

    let client = Client::new().await.context(KubeClientCreateSnafu)?;

    // If the user wants to list stacklets from all namespaces, we use `None`.
    // `None` indicates that don't want to list stacklets scoped to only ONE
    // namespace.
    let stacklets = list_stacklets(&client, Some(&args.namespaces.namespace))
        .await
        .context(StackletListSnafu)?;

    if stacklets.is_empty() {
        let mut result = Cli::result();

        result
            .with_command_hint(
                "stackablectl stack install <STACK_NAME>",
                "install a complete stack",
            )
            .with_command_hint(
                "stackablectl demo install <DEMO_NAME>",
                "install an end-to-end demo",
            )
            .with_output("No stacklets found");

        return Ok(result.render());
    }

    match args.output_type {
        OutputType::Plain | OutputType::Table => {
            let (arrangement, preset) = match args.output_type {
                OutputType::Plain => (ContentArrangement::Disabled, NOTHING),
                _ => (ContentArrangement::Dynamic, UTF8_FULL),
            };

            // The main table displays all installed (and discovered) stacklets
            // and their condition.
            let mut table = Table::new();
            table
                .set_header(vec![
                    "PRODUCT",
                    "NAME",
                    "NAMESPACE",
                    "ENDPOINTS",
                    "CONDITIONS",
                ])
                .set_content_arrangement(arrangement)
                .load_preset(preset);

            let mut error_list = Vec::new();
            let mut error_index = 1;

            let max_endpoint_name_length = match args.output_type {
                OutputType::Plain => 0,
                _ => stacklets
                    .iter()
                    .flat_map(|s| &s.endpoints)
                    .map(|(endpoint_name, _)| endpoint_name.len())
                    .max()
                    .unwrap_or_default(),
            };

            for stacklet in stacklets {
                let ConditionOutput { summary, errors } =
                    render_conditions(stacklet.conditions, &mut error_index);

                let endpoints = stacklet
                    .endpoints
                    .iter()
                    .map(|(name, url)| {
                        format!("{name:width$} {url}", width = max_endpoint_name_length + 1)
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                table.add_row(vec![
                    stacklet.product,
                    stacklet.name,
                    stacklet.namespace.unwrap_or_default(),
                    endpoints,
                    summary,
                ]);

                if let Some(err) = render_errors(errors) {
                    error_list.push(err)
                }
            }

            let mut result = Cli::result();

            result
                .with_command_hint(
                    "stackablectl stacklet credentials [OPTIONS] <PRODUCT_NAME> <STACKLET_NAME>",
                    "display credentials for deployed stacklets",
                )
                .with_output(format!(
                    "{table}{errors}",
                    errors = if !error_list.is_empty() {
                        format!("\n\n{error_list}", error_list = error_list.join("\n"))
                    } else {
                        "".into()
                    }
                ));

            Ok(result.render())
        }
        OutputType::Json => serde_json::to_string(&stacklets).context(SerializeJsonOutputSnafu),
        OutputType::Yaml => yaml_serde::to_string(&stacklets).context(SerializeYamlOutputSnafu),
    }
}

#[instrument(skip_all, fields(indicatif.pb_show = true))]
async fn credentials_cmd(args: &StackletCredentialsArgs) -> Result<String, CmdError> {
    info!("Displaying stacklet credentials");
    Span::current().pb_set_message("Fetching stacklet information");

    let client = Client::new().await.context(KubeClientCreateSnafu)?;

    match get_credentials_for_product(
        &client,
        &args.namespace,
        &args.stacklet_name,
        &args.product_name,
    )
    .await
    .context(StackletCredentialsSnafu)?
    {
        Some(credentials) => {
            let mut table = Table::new();

            table
                .set_content_arrangement(ContentArrangement::Dynamic)
                .load_preset(NOTHING)
                .add_row(vec!["USERNAME", &credentials.username])
                .add_row(vec!["PASSWORD", &credentials.password]);

            let output = format!(
                "Credentials for {product_name} ({stacklet_name}) in namespace {namespace:?}:",
                product_name = args.product_name,
                stacklet_name = args.stacklet_name,
                namespace = args.namespace
            );

            Ok(format!("{output}\n\n{table}"))
        }
        None => Ok("No credentials".into()),
    }
}

pub struct ConditionOutput {
    summary: String,
    errors: Vec<String>,
}

/// Renders conditions for a single stacklet / product. It returns a
/// concatenated string of conditions (which are colored green and red) to
/// display next to each listed stacklet in the table. Additionally, it also
/// returns a list of errors to be displayed underneath the stacklet table.
fn render_conditions(
    product_conditions: Vec<DisplayCondition>,
    error_index: &mut usize,
) -> ConditionOutput {
    let mut conditions = Vec::new();
    let mut errors = Vec::new();

    for cond in product_conditions {
        conditions.push(color_condition(&cond.condition, cond.is_good, *error_index));

        if let Some(error) = render_condition_error(cond.message, cond.is_good, *error_index) {
            errors.push(error);
            *error_index += 1;
        };
    }

    ConditionOutput {
        summary: conditions.join(", "),
        errors,
    }
}

/// Renders one condition and determines if it is an error (not good). If this
/// is the case, it get colored red and is returned.
fn render_condition_error(
    message: Option<String>,
    is_good: Option<bool>,
    error_index: usize,
) -> Option<String> {
    if !is_good.unwrap_or(true) {
        let message = message.unwrap_or("-".into());
        return Some(format!("[{error_index}]: {message}"));
    }

    None
}

// TODO (Techassi): Add back color support
/// Adds an error index to the output.
fn color_condition(condition: &str, is_good: Option<bool>, error_index: usize) -> String {
    match is_good {
        Some(is_good) => {
            if is_good {
                condition.to_owned()
            } else {
                format!("{condition}: See [{error_index}]")
            }
        }
        None => condition.to_owned(),
    }
}

/// Renders multiple errors (of one stacklet)
fn render_errors(errors: Vec<String>) -> Option<String> {
    if errors.is_empty() {
        None
    } else if errors.len() == 1 {
        Some(errors[0].clone())
    } else {
        Some(format!("{errors}\n---\n", errors = errors.join("\n")))
    }
}
