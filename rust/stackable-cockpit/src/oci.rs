use std::collections::HashMap;

use serde::Deserialize;
use snafu::{OptionExt, ResultExt, Snafu};
use tracing::{debug, instrument};
use url::Url;
use urlencoding::encode;

use crate::{
    constants::{
        HELM_OCI_BASE, HELM_REPO_NAME_DEV, HELM_REPO_NAME_STABLE, HELM_REPO_NAME_TEST,
        OCI_INDEX_PAGE_SIZE,
    },
    utils::chartsource::{ChartSourceEntry, ChartSourceMetadata},
};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("cannot get repositories"))]
    GetRepositories { source: reqwest::Error },

    #[snafu(display("cannot parse repositories"))]
    ParseRepositories { source: reqwest::Error },

    #[snafu(display("cannot get artifacts"))]
    GetArtifacts { source: reqwest::Error },

    #[snafu(display("cannot parse artifacts"))]
    ParseArtifacts { source: reqwest::Error },

    #[snafu(display("unexpected OCI repository name"))]
    UnexpectedOciRepositoryName,

    #[snafu(display("cannot resolve path segments"))]
    GetPathSegments,

    #[snafu(display("failed to parse URL"))]
    UrlParse { source: url::ParseError },
}

/// Identifies an operator-specific root folder in the repository e.g.
/// ```json
/// {
///   name: "sdp-charts/airflow-operator"
/// }
/// ```
#[derive(Deserialize, Debug)]
struct OciRepository {
    pub name: String,
}

/// Identifies an image tag e.g.
/// ```json
/// {
///   name: "24.11.1-rc1"
/// }
/// ```
#[derive(Deserialize, Debug)]
pub struct Tag {
    pub name: String,
}

/// Identifies an image artifact with its digest and tags e.g.
/// ```json
/// {
///   digest: "sha256:e80a4b1e004f90dee0321f817871c4a225369b89efdc17c319595263139364b5",
///   tags: [
///     {
///       name: "0.0.0-pr569"
///     }
///   ])
/// }
/// ```
#[derive(Deserialize, Debug)]
pub struct Artifact {
    pub digest: String,
    pub tags: Option<Vec<Tag>>,
}

trait OciUrlExt {
    fn oci_artifacts_page(
        &self,
        project_name: &str,
        repository_name: &str,
        page_size: usize,
        page: usize,
    ) -> Result<Url, Error>;
}

impl OciUrlExt for Url {
    fn oci_artifacts_page(
        &self,
        project_name: &str,
        repository_name: &str,
        page_size: usize,
        page: usize,
    ) -> Result<Url, Error> {
        let encoded_project = encode(project_name);
        let encoded_repo = encode(repository_name);

        let mut url = self.clone();
        url.path_segments_mut()
            .map_err(|_| Error::GetPathSegments)?
            .extend(&[
                "projects",
                &encoded_project,
                "repositories",
                &encoded_repo,
                "artifacts",
            ]);

        url.query_pairs_mut()
            .append_pair("page_size", &page_size.to_string())
            .append_pair("page", &page.to_string());

        Ok(url)
    }
}

// TODO (@NickLarsenNZ): Look into why a HashMap is used here when the key is inside each entry in the value
#[instrument]
pub async fn get_oci_index<'a>() -> Result<HashMap<&'a str, ChartSourceMetadata>, Error> {
    let mut source_index_files: HashMap<&str, ChartSourceMetadata> = HashMap::new();

    // initialize map
    for repo_name in [
        HELM_REPO_NAME_STABLE,
        HELM_REPO_NAME_TEST,
        HELM_REPO_NAME_DEV,
    ] {
        source_index_files.insert(
            repo_name,
            ChartSourceMetadata {
                entries: HashMap::new(),
            },
        );
    }
    let base_url = format!("https://{HELM_OCI_BASE}/api/v2.0");

    // fetch all operators
    let url = format!(
        "{base_url}/repositories?page_size={page_size}&q=name=~sdp-charts/",
        page_size = 100
    );

    // reuse connections
    let client = reqwest::Client::new();

    let repositories: Vec<OciRepository> = client
        .get(&url)
        .send()
        .await
        .context(GetRepositoriesSnafu)?
        .json()
        .await
        .context(ParseRepositoriesSnafu)?;

    debug!(
        count = repositories.len(),
        "Received response for OCI repositories"
    );

    for repository in &repositories {
        // fetch all artifacts pro operator
        // NOTE (@NickLarsenNZ): I think repository_name should be helm_chart_name.
        let (project_name, repository_name) = repository
            .name
            .split_once('/')
            .context(UnexpectedOciRepositoryNameSnafu)?;

        tracing::trace!(project_name, repository_name, "OCI repository parts");

        let mut artifacts = Vec::new();
        let mut page = 1;

        loop {
            let root = Url::parse(base_url.as_str()).context(UrlParseSnafu)?;
            let url =
                root.oci_artifacts_page(project_name, repository_name, OCI_INDEX_PAGE_SIZE, page)?;
            let artifacts_page = client
                .get(url)
                .send()
                .await
                .context(GetArtifactsSnafu)?
                .json::<Vec<Artifact>>()
                .await
                .context(ParseArtifactsSnafu)?;
            let count = artifacts_page.len();
            artifacts.extend(artifacts_page);
            if count < OCI_INDEX_PAGE_SIZE {
                break;
            }
            page += 1;
        }

        for artifact in &artifacts {
            if let Some(release_artifact) =
                artifact.tags.as_ref().and_then(|tags| tags.iter().next())
            {
                let release_version = release_artifact
                    .name
                    .replace("-arm64", "")
                    .replace("-amd64", "");

                tracing::trace!(repository_name, release_version, "OCI resolved artifact");

                let entry = ChartSourceEntry {
                    name: repository_name.to_string(),
                    version: release_version.to_string(),
                };

                match release_version.as_str() {
                    // Ignore "artifacthub.io" tags which don't point to an actual Helm Chart, but
                    // to an ArtifactHub metadata file.
                    "artifacthub.io" => continue,
                    "0.0.0-dev" => {
                        if let Some(repo) = source_index_files.get_mut(HELM_REPO_NAME_DEV) {
                            repo.entries
                                .entry(repository_name.to_string())
                                .or_default()
                                .push(entry)
                        }
                    }
                    version if version.contains("-pr") => {
                        if let Some(repo) = source_index_files.get_mut(HELM_REPO_NAME_TEST) {
                            repo.entries
                                .entry(repository_name.to_string())
                                .or_default()
                                .push(entry)
                        }
                    }
                    _ => {
                        if let Some(repo) = source_index_files.get_mut(HELM_REPO_NAME_STABLE) {
                            repo.entries
                                .entry(repository_name.to_string())
                                .or_default()
                                .push(entry)
                        }
                    }
                }
            }
        }
    }
    Ok(source_index_files)
}
