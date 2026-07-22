use std::{env, path::Path, sync::Arc};

use clap::{Parser, Subcommand, ValueEnum};
use directories::ProjectDirs;
use snafu::{OptionExt, ResultExt, Snafu};
use stackable_cockpit::{
    constants::{HELM_REPO_NAME_DEV, HELM_REPO_NAME_STABLE, HELM_REPO_NAME_TEST},
    helm,
    platform::operator::{
        ChartSourceType, listener_operator::determine_and_store_listener_class_preset,
    },
    utils::path::{
        IntoPathOrUrl, IntoPathsOrUrls, ParsePathsOrUrls, PathOrUrl, PathOrUrlParseError,
    },
    xfer::{self, cache::Settings},
};
use tracing::{Level, instrument};
use tracing_indicatif::indicatif_eprintln;

use crate::{
    args::{CommonFileArgs, CommonOperatorConfigsArgs, CommonRepoArgs},
    cmds::{cache, completions, debug, demo, operator, release, stack, stacklet, version},
    config::UserConfig,
    constants::{
        DEMOS_REPOSITORY_DEMOS_SUBPATH, DEMOS_REPOSITORY_STACKS_SUBPATH, DEMOS_REPOSITORY_URL_BASE,
        ENV_KEY_DEMO_FILES, ENV_KEY_RELEASE_FILES, ENV_KEY_STACK_FILES, REMOTE_RELEASE_FILE,
        USER_DIR_APPLICATION_NAME, USER_DIR_ORGANIZATION_NAME, USER_DIR_QUALIFIER,
    },
    output::{ErrorContext, Output, ResultContext},
    release_check,
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("failed to execute operator (sub)command"))]
    Operator { source: operator::CmdError },

    #[snafu(display("failed to execute release (sub)command"))]
    Release { source: release::CmdError },

    #[snafu(display("failed to execute stack (sub)command"))]
    Stack { source: stack::CmdError },

    #[snafu(display("failed to execute stacklet (sub)command"))]
    Stacklet { source: stacklet::CmdError },

    #[snafu(display("failed to execute demo (sub)command"))]
    Demo { source: demo::CmdError },

    #[snafu(display("failed to execute completions (sub)command"))]
    Completions { source: completions::CmdError },

    #[snafu(display("failed to execute cache (sub)command"))]
    Cache { source: cache::CmdError },

    #[snafu(display("failed to execute debug (sub)command"))]
    Debug {
        #[snafu(source(from(debug::CmdError, Box::new)))]
        source: Box<debug::CmdError>,
    },

    #[snafu(display("failed to execute version (sub)command"))]
    Version { source: version::CmdError },

    #[snafu(display("failed to add Helm repositories"))]
    AddHelmRepos { source: helm::Error },

    #[snafu(display("failed to retrieve cache settings"))]
    RetrieveCacheSettings { source: CacheSettingsError },

    #[snafu(display("failed to auto-purge cache"))]
    AutoPurgeCache { source: xfer::Error },

    #[snafu(display("failed to initialize transfer client"))]
    InitializeTransferClient { source: xfer::Error },

    #[snafu(display("failed to retrieve XDG directories"))]
    RetrieveXdgDirectories,
}

#[derive(Debug, Snafu)]
#[snafu(module)]
pub enum CacheSettingsError {
    #[snafu(display("unable to resolve user directories"))]
    UserDir,
}

#[derive(Debug, Parser)]
#[command(author, version, about, propagate_version = true)]
pub struct Cli {
    /// Log level this application uses
    #[arg(short, long, global = true)]
    pub log_level: Option<Level>,

    /// Do not cache the remote (default) demo, stack and release files
    #[arg(
        long,
        global = true,
        long_help = "Do not cache the remote (default) demo, stack and release files

Cached files are saved at '$XDG_CACHE_HOME/stackablectl', which is usually
'$HOME/.cache/stackablectl' when not explicitly set."
    )]
    pub no_cache: bool,

    #[command(flatten)]
    pub files: CommonFileArgs,

    #[command(flatten)]
    pub repos: CommonRepoArgs,

    #[command(flatten)]
    pub operator_configs: CommonOperatorConfigsArgs,

    #[command(subcommand)]
    pub subcommand: Command,
}

impl Cli {
    /// Returns a list of demo files, consisting of entries which are either a path or URL. The list of files combines
    /// the default demo file URL constructed from [`DEMOS_REPOSITORY_URL_BASE`] and the provided branch, files provided
    /// by the ENV variable [`ENV_KEY_DEMO_FILES`], and lastly, files provided by the CLI argument `--demo-file`.
    pub fn get_demo_files(&self, branch: &str) -> Result<Vec<PathOrUrl>, PathOrUrlParseError> {
        let branch_url =
            format!("{DEMOS_REPOSITORY_URL_BASE}/{branch}/{DEMOS_REPOSITORY_DEMOS_SUBPATH}");

        let mut files = get_files(&branch_url, ENV_KEY_DEMO_FILES)?;

        let arg_files = self.files.demo_files.clone().into_paths_or_urls()?;
        files.extend(arg_files);

        Ok(files)
    }

    /// Returns a list of stack files, consisting of entries which are either a path or URL. The list of files combines
    /// the default stack file URL constructed from [`DEMOS_REPOSITORY_URL_BASE`] and the provided branch, files provided
    /// by the ENV variable [`ENV_KEY_STACK_FILES`], and lastly, files provided by the CLI argument `--stack-file`.
    pub fn get_stack_files(&self, branch: &str) -> Result<Vec<PathOrUrl>, PathOrUrlParseError> {
        let branch_url =
            format!("{DEMOS_REPOSITORY_URL_BASE}/{branch}/{DEMOS_REPOSITORY_STACKS_SUBPATH}");

        let mut files = get_files(&branch_url, ENV_KEY_STACK_FILES)?;

        let arg_files = self.files.stack_files.clone().into_paths_or_urls()?;
        files.extend(arg_files);

        Ok(files)
    }

    /// Returns a list of release files, consisting of entries which are either a path or URL. The list of files
    /// combines the default demo file URL, [`REMOTE_RELEASE_FILE`], files provided by the ENV variable
    /// [`ENV_KEY_RELEASE_FILES`], and lastly, files provided by the CLI argument `--release-file`.
    pub fn get_release_files(&self) -> Result<Vec<PathOrUrl>, PathOrUrlParseError> {
        let mut files = get_files(REMOTE_RELEASE_FILE, ENV_KEY_RELEASE_FILES)?;

        let arg_files = self.files.release_files.clone().into_paths_or_urls()?;
        files.extend(arg_files);

        Ok(files)
    }

    pub fn get_values_file(&self) -> Result<Option<PathOrUrl>, PathOrUrlParseError> {
        if let Some(operator_values) = self.files.operator_values.clone() {
            Ok(Some(operator_values.into_path_or_url()?))
        } else {
            Ok(None)
        }
    }

    /// Adds the default (or custom) Helm repository URLs. Internally this calls the Helm SDK written in Go through the
    /// `go-helm-wrapper`.
    pub fn add_helm_repos(&self) -> Result<(), helm::Error> {
        tracing::info!("Add Helm repos");

        // Stable repository
        helm::add_repo(HELM_REPO_NAME_STABLE, &self.repos.helm_repo_stable)?;

        // Test repository
        helm::add_repo(HELM_REPO_NAME_TEST, &self.repos.helm_repo_test)?;

        // Dev repository
        helm::add_repo(HELM_REPO_NAME_DEV, &self.repos.helm_repo_dev)?;

        Ok(())
    }

    fn cache_settings(&self, cache_directory: &Path) -> Result<Settings, CacheSettingsError> {
        if self.no_cache {
            tracing::debug!("Cache disabled");
            Ok(Settings::disabled())
        } else {
            tracing::debug!(
                cache_directory = %cache_directory.to_string_lossy(),
                "Setting cache directory"
            );
            Ok(Settings::disk(cache_directory))
        }
    }

    fn xdg_directories() -> Result<ProjectDirs, Error> {
        ProjectDirs::from(
            USER_DIR_QUALIFIER,
            USER_DIR_ORGANIZATION_NAME,
            USER_DIR_APPLICATION_NAME,
        )
        .context(RetrieveXdgDirectoriesSnafu)
    }

    #[instrument(skip_all)]
    pub async fn run(self) -> Result<String, Error> {
        // FIXME (Techassi): There might be a better way to handle this with
        // the match later in this function.

        // Add Helm repos only when required
        match &self.subcommand {
            Command::Completions(_) => (),
            Command::Cache(_) => (),
            _ => self.add_helm_repos().context(AddHelmReposSnafu)?,
        }

        let xdg_directories = Cli::xdg_directories()?;
        // TODO (@Techassi): Move this file name to a constant
        let user_config_path = xdg_directories.config_dir().join("config.toml");

        let user_config = UserConfig::from_file_or_default(user_config_path).unwrap();

        let cache_settings = self
            .cache_settings(xdg_directories.cache_dir())
            .context(RetrieveCacheSettingsSnafu)?;
        let transfer_client = xfer::Client::new(cache_settings)
            .await
            .context(InitializeTransferClientSnafu)?;

        // Only run the cache auto-purge when the user executes ANY command other than the cache
        // commands.
        if !matches!(self.subcommand, Command::Cache(_)) {
            transfer_client
                .auto_purge()
                .await
                .context(AutoPurgeCacheSnafu)?;
        }

        // Make transfer client sharable across multiple futures/threads
        let transfer_client = Arc::new(transfer_client);

        determine_and_store_listener_class_preset(
            self.operator_configs.listener_class_preset.as_ref(),
        )
        .await;

        // Only run the version check in the background if the user runs ANY other command than
        // the version command and the check isn't disabled via the user config. Also only report
        // if the current version is outdated.
        let check_version =
            !matches!(self.subcommand, Command::Version(_)) && user_config.version.check_enabled;
        let release_check_future =
            release_check::version_notice_output(transfer_client.clone(), check_version, true);
        let release_check_future =
            tokio::time::timeout(std::time::Duration::from_secs(2), release_check_future);

        #[rustfmt::skip]
        let command_future = async move {
            match self.subcommand {
                Command::Operator(ref args) => args.run(&self, transfer_client).await.context(OperatorSnafu),
                Command::Release(ref args) => args.run(&self, transfer_client).await.context(ReleaseSnafu),
                Command::Stack(ref args) => args.run(&self, transfer_client).await.context(StackSnafu),
                Command::Stacklet(ref args) => args.run().await.context(StackletSnafu),
                Command::Demo(ref args) => args.run(&self, transfer_client).await.context(DemoSnafu),
                Command::Completions(ref args) => args.run().context(CompletionsSnafu),
                Command::Cache(ref args) => args.run(transfer_client).await.context(CacheSnafu),
                Command::ExperimentalDebug(ref args) => args.run().await.context(DebugSnafu),
                Command::Version(ref args) => args.run(transfer_client).await.context(VersionSnafu),
            }
        };

        // Run the version check and the actual command in parallel and not sequentially. This is
        // done to not abort/stall execution when the version check couldn't be performed (because
        // of network issues for example). We also optimistically run the version check and don't
        // hard-error below when the check failed.
        let (release_check_result, command_result) =
            tokio::join!(release_check_future, command_future);

        // NOTE (@Techassi): This is freaking ugly (I'm sorry) but there seems to be no other better
        // way to achieve what we want without reworking the entire output handling/rendering
        // mechanism.
        // FIXME (@Techassi): This currently messes up any structured output. This is also not
        // trivially solved as explained above.
        match command_result {
            Ok(command_output) => {
                let output = if let Ok(Ok(Some(release_check_output))) = release_check_result {
                    format!("{command_output}\n\n{release_check_output}")
                } else {
                    command_output
                };

                Ok(output)
            }
            Err(err) => {
                if let Ok(Ok(Some(release_check_output))) = release_check_result {
                    indicatif_eprintln!("{release_check_output}\n");
                }

                Err(err)
            }
        }
    }

    // Output utility functions
    pub fn result() -> Output<ResultContext> {
        Output::new(ResultContext::default(), true).expect("Failed to create output renderer")
    }

    pub fn error() -> Output<ErrorContext> {
        Output::new(ErrorContext::default(), true).expect("Failed to create output renderer")
    }

    pub fn chart_type(&self) -> ChartSourceTypeArg {
        self.repos.chart_source.clone()
    }
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Interact with single operator instead of the full platform
    #[command(alias("op"))]
    Operator(operator::OperatorArgs),

    /// Interact with all operators of the platform which are released together
    #[command(alias("re"))]
    Release(release::ReleaseArgs),

    /// Interact with stacks, which are ready-to-use product combinations
    #[command(alias("st"))]
    Stack(stack::StackArgs),

    /// Interact with deployed stacklets, which are bundles of resources and
    /// containers required to run the product.
    #[command(aliases(["stl", "sl"]))]
    #[command(
        long_about = "Interact with deployed stacklets, which are bundles of resources and containers
required to run the product.

Each stacklet consists of init containers, app containers, sidecar containers
and additional Kubernetes resources like StatefulSets, ConfigMaps, Services and
CRDs."
    )]
    Stacklet(stacklet::StackletArgs),

    /// Interact with demos, which are end-to-end usage demonstrations of the Stackable data platform
    Demo(demo::DemoArgs),

    /// Generate shell completions for this tool
    #[command(alias("comp"))]
    Completions(completions::CompletionsArgs),

    /// Interact with locally cached files
    Cache(cache::CacheArgs),

    /// EXPERIMENTAL: Launch a debug container for a Pod
    #[command(long_about = "EXPERIMENTAL: Launch a debug container for a Pod.

This container will have access to the same data volumes as the primary container.")]
    ExperimentalDebug(debug::DebugArgs),

    /// Retrieve version data of the stackablectl installation
    Version(version::VersionArguments),
}

#[derive(Clone, Debug, Default, ValueEnum)]
pub enum OutputType {
    /// Print output formatted as plain text
    Plain,

    /// Print output formatted as a table
    #[default]
    Table,

    /// Print output formatted as JSON
    Json,

    /// Print output formatted as YAML
    Yaml,
}

/// Returns a list of paths or urls based on the default (remote) file and
/// files provided via the env variable.
fn get_files(default_file: &str, env_key: &str) -> Result<Vec<PathOrUrl>, PathOrUrlParseError> {
    let mut files: Vec<PathOrUrl> = vec![default_file.into_path_or_url()?];

    let env_files = match env::var(env_key) {
        Ok(env_files) => env_files.parse_paths_or_urls()?,
        Err(_) => vec![],
    };
    files.extend(env_files);

    Ok(files)
}

/// Enum used for resolving the argument for chart source type. This will be
/// mapped to ChartSourceType (see below): the reason why we don't have one
/// enum is to avoid having to add clap dependencies to stackable-cockpit
/// for the ValueEnum macro.
#[derive(Clone, Debug, Default, ValueEnum)]
pub enum ChartSourceTypeArg {
    /// OCI registry
    #[default]
    OCI,

    /// index.yaml-based repositories: resolution (dev, test, stable) is based on the version and thus will be operator-specific
    Repo,
}

impl From<ChartSourceTypeArg> for ChartSourceType {
    /// Resolves the enum used by clap/arg-resolution to the core type used in
    /// stackable-cockpit. For the (index.yaml-based) repo case this core type cannot be
    /// decorated with meaningful information as that would be operator-specific
    /// i.e. we cannot resolve *which* (index.yaml-based) repo to use until we have inspected
    /// the operator version. Hence just a simple mapping.
    fn from(cli_enum: ChartSourceTypeArg) -> Self {
        match cli_enum {
            ChartSourceTypeArg::OCI => ChartSourceType::OCI,
            ChartSourceTypeArg::Repo => ChartSourceType::Repo,
        }
    }
}
