use std::{str::FromStr, sync::Arc};

use semver::Version;
use serde::Deserialize;
use snafu::{OptionExt, ResultExt, Snafu};
use stackable_cockpit::{utils::path::PathOrUrl, xfer};

use crate::built_info::PKG_SEMVER;

const URL: &str = "https://api.github.com/repos/stackabletech/stackablectl/releases";
const PREFIX: &str = "stackablectl-";

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("failed to retrieve list of releases"))]
    RetrieveReleases { source: xfer::Error },

    #[snafu(display("failed to find latest release"))]
    FindLatestRelease,

    #[snafu(display("failed to parse {input} as semantic version"))]
    ParseVersion {
        source: semver::Error,
        input: String,
    },
}

#[derive(Debug, Deserialize)]
pub struct Release {
    pub name: String,
    pub prerelease: bool,
    pub draft: bool,
}

async fn fetch_latest_release_version_from_github(
    client: Arc<xfer::Client>,
) -> Result<Version, Error> {
    let url = PathOrUrl::from_str(URL).expect("constant URL must be a valid URL");

    let releases: Vec<Release> = client
        .get(&url, &xfer::processor::Json::default())
        .await
        .context(RetrieveReleasesSnafu)?;

    // We assume the list of releases is ordered (newest to oldest).
    // If this ever changes, sorting needs to be done here instead.
    let latest_release = releases
        .into_iter()
        // Filter out any draft and prerelease releases.
        .filter(|release| !release.draft && !release.prerelease)
        // Find a release starting with 'stackablectl-'.
        .find(|release| release.name.starts_with(PREFIX))
        .context(FindLatestReleaseSnafu)?;

    let version = latest_release.name.trim_start_matches(PREFIX).to_owned();
    let version = Version::from_str(&version).context(ParseVersionSnafu { input: version })?;

    Ok(version)
}

pub async fn version_notice_output(
    client: Arc<xfer::Client>,
    run_check: bool,
    only_output_outdated: bool,
) -> Result<Option<String>, Error> {
    if !run_check {
        return Ok(None);
    }

    let latest_version = fetch_latest_release_version_from_github(client).await?;
    let current_version = &*PKG_SEMVER;

    let output = if &latest_version > current_version {
        Some(format!(
            "The current stackablectl version ({current_version}) is out-of-date. The latest version available is {latest_version}"
        ))
    } else if !only_output_outdated {
        Some(format!(
            "The current stackablectl version ({current_version}) is up-to-date"
        ))
    } else {
        None
    };

    Ok(output)
}
