use std::env;

use serde_yaml::{Mapping, Value};
use snafu::{ResultExt, Snafu};
use stackable_cockpit::{
    constants::{HELM_REPO_NAME_DEV, HELM_REPO_NAME_STABLE, HELM_REPO_NAME_TEST},
    utils::path::PathOrUrl,
    xfer::{self, processor::Yaml},
};

use crate::constants::{HELM_REPO_URL_DEV, HELM_REPO_URL_STABLE, HELM_REPO_URL_TEST};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("failed to open or transfer values file '{path}'"))]
    FileTransfer { source: xfer::Error, path: String },

    #[snafu(display("operator values file '{path}' must be a YAML mapping at the top level"))]
    InvalidValueType { path: String },

    #[snafu(display(
        "value for key '{key}' in operator values file '{path}' must be a YAML mapping"
    ))]
    InvalidEntryType { key: String, path: String },
}

#[derive(Debug, Snafu)]
#[snafu(display("Invalid Helm repo name ({name}), cannot resolve to repo URL"))]
pub struct InvalidRepoNameError {
    name: String,
}

/// This returns the Helm repository URL based on the repo name. If the provided
/// repo name is not recognized (invalid), an [`InvalidRepoNameError`] is
/// returned.
pub fn helm_repo_name_to_repo_url<'a, T>(repo_name: T) -> Result<&'a str, InvalidRepoNameError>
where
    T: AsRef<str>,
{
    let repo_name = repo_name.as_ref();

    match repo_name {
        HELM_REPO_NAME_STABLE => Ok(HELM_REPO_URL_STABLE),
        HELM_REPO_NAME_TEST => Ok(HELM_REPO_URL_TEST),
        HELM_REPO_NAME_DEV => Ok(HELM_REPO_URL_DEV),
        _ => Err(InvalidRepoNameError {
            name: repo_name.to_string(),
        }),
    }
}

/// Returns whether the application should use colored output based on the user
/// requested output and the `NO_COLOR` env variable. It currently does not
/// factor in terminal support.
pub fn use_colored_output(use_color: bool) -> bool {
    use_color && env::var_os("NO_COLOR").is_none()
}

/// Loads operator helm values from a YAML file.
///
/// The file should contain a YAML mapping of operator names to their helm values.
/// Use YAML anchors and aliases to share values across operators:
/// ```yaml
/// airflow-operator:
///   tolerations: &default-tolerations
///     - key: "example"
///       operator: "Exists"
///       effect: "NoSchedule"
///   podAnnotations:
///     example.com/team: "data-engineering"
/// zookeeper-operator:
///   tolerations: *default-tolerations
///   podAnnotations:
///     example.com/team: "platform"
/// ```
pub async fn load_operator_values(
    values_file: Option<&PathOrUrl>,
    transfer_client: &xfer::Client,
) -> Result<Mapping, Error> {
    let file = match values_file {
        Some(file) => file,
        None => return Ok(Mapping::new()),
    };

    let path = match file {
        PathOrUrl::Path(p) => p.display().to_string(),
        PathOrUrl::Url(u) => u.to_string(),
    };

    let value = transfer_client
        .get(file, &Yaml::<Value>::default())
        .await
        .context(FileTransferSnafu { path: path.clone() })?;

    let mapping = match value {
        Value::Mapping(mapping) => mapping,
        _ => return InvalidValueTypeSnafu { path: path.clone() }.fail(),
    };

    for (key, value) in &mapping {
        if !value.is_mapping() {
            return InvalidEntryTypeSnafu {
                key: key.as_str().unwrap_or("<non-string key>").to_string(),
                path: path.clone(),
            }
            .fail();
        }
    }

    Ok(mapping)
}
