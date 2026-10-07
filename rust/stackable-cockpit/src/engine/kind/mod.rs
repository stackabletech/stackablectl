use std::process::Stdio;

use snafu::{OptionExt, ResultExt, Snafu, ensure};
use tokio::{io::AsyncWriteExt, process::Command};
use tracing::{debug, info, instrument};

use crate::{
    engine::{
        docker::{self, check_if_docker_is_running},
        kind::config::Config,
    },
    utils::check::binaries_present_with_name,
};

mod config;

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("failed to pipe kind config using stdin"))]
    PipeConfigStdin { source: std::io::Error },

    #[snafu(display("failed to obtain stdin handle"))]
    ObtainStdinHandle,

    #[snafu(display("failed to start kind command"))]
    CommandFailedToStart { source: std::io::Error },

    #[snafu(display("failed to run kind command"))]
    CommandFailedToRun { source: std::io::Error },

    #[snafu(display("failed to successfully run kind command ({error})"))]
    CommandErroredOut { error: String },

    #[snafu(display("missing required binary {binary:?}"))]
    MissingBinary { binary: String },

    #[snafu(display("failed to determine if Docker is running"))]
    DockerCheckCommand { source: docker::Error },

    #[snafu(display("failed to covert kind config to YAML"))]
    ConfigSerialization { source: serde_yaml::Error },
}

#[derive(Debug)]
pub struct Cluster {
    cp_node_count: usize,
    node_count: usize,
    name: String,
}

impl Cluster {
    /// Create a new kind cluster. This will NOT yet create the cluster on the
    /// system, but instead will return a data structure representing the
    /// cluster. To actually create the cluster, the `create` method must be
    /// called.
    pub fn new(node_count: usize, cp_node_count: usize, name: String) -> Self {
        Self {
            cp_node_count,
            node_count,
            name,
        }
    }

    /// Create a new local cluster by calling the kind binary.
    #[instrument(skip_all)]
    pub async fn create(&self) -> Result<()> {
        info!("Creating local cluster using kind");

        // Check if required binaries are present
        if let Some(binary) = binaries_present_with_name(&["docker", "kind"]) {
            return Err(Error::MissingBinary { binary });
        }

        // Check if Docker is running
        check_if_docker_is_running()
            .await
            .context(DockerCheckCommandSnafu)?;

        debug!("Creating kind cluster config");
        let config = Config::new(self.node_count, self.cp_node_count);
        let config_string = serde_yaml::to_string(&config).context(ConfigSerializationSnafu)?;

        debug!("Creating kind cluster");
        let mut kind_cmd = Command::new("kind")
            .args(["create", "cluster"])
            .args(["--name", self.name.as_str()])
            .args(["--config", "-"])
            .stdin(Stdio::piped())
            .spawn()
            .context(CommandFailedToStartSnafu)?;

        // Try to obtain stdin handle
        let mut stdin = kind_cmd.stdin.take().context(ObtainStdinHandleSnafu)?;

        // Pipe in config
        stdin
            .write_all(config_string.as_bytes())
            .await
            .context(PipeConfigStdinSnafu)?;

        // Write the piped in data
        stdin.flush().await.context(PipeConfigStdinSnafu)?;
        drop(stdin);

        kind_cmd.wait().await.context(CommandFailedToRunSnafu)?;
        Ok(())
    }

    /// Creates a kind cluster if it doesn't exist already.
    #[instrument(skip_all)]
    pub async fn create_if_not_exists(&self) -> Result<()> {
        info!("Creating cluster if it doesn't exist using kind");

        if Self::check_if_cluster_exists(&self.name).await? {
            return Ok(());
        }

        self.create().await
    }

    /// Retrieve the cluster node count
    pub fn get_node_count(&self) -> usize {
        self.node_count
    }

    /// Retrieve the cluster name
    pub fn get_name(&self) -> &String {
        &self.name
    }

    /// Check if a kind cluster with the provided name already exists.
    #[instrument(skip_all)]
    async fn check_if_cluster_exists(cluster_name: &str) -> Result<bool> {
        debug!("Checking if kind cluster exists");

        let output = Command::new("kind")
            .args(["get", "clusters"])
            .output()
            .await
            .context(CommandFailedToRunSnafu)?;

        ensure!(
            output.status.success(),
            CommandErroredOutSnafu {
                error: String::from_utf8_lossy(&output.stderr)
            }
        );

        let output = String::from_utf8_lossy(&output.stdout);
        Ok(output.lines().any(|name| name == cluster_name))
    }
}
