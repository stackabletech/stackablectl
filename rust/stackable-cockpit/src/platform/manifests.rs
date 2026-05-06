use std::collections::HashMap;

use snafu::{ResultExt, Snafu};
use stackable_operator::kvp::Labels;
use tracing::{Span, debug, info, instrument};
use tracing_indicatif::span_ext::IndicatifSpanExt as _;

use crate::{
    PROGRESS_BAR_STYLE,
    common::manifest::ManifestSpec,
    helm,
    utils::{
        k8s::{self, Client},
        path::{IntoPathOrUrl, PathOrUrlParseError},
    },
    xfer::{
        self,
        processor::{Processor, Template, Yaml},
    },
};

#[derive(Debug, Snafu)]
pub enum Error {
    /// This error indicates that parsing a string into a path or URL failed.
    #[snafu(display("failed to parse {path_or_url:?} as path/url"))]
    ParsePathOrUrl {
        source: PathOrUrlParseError,
        path_or_url: String,
    },

    /// This error indicates that receiving remote content failed.
    #[snafu(display("failed to receive remote content"))]
    FileTransfer { source: xfer::Error },

    /// This error indicates that the Helm wrapper failed to add the Helm
    /// repository.
    #[snafu(display("failed to add Helm repository {repo_name}"))]
    AddHelmRepository {
        source: helm::Error,
        repo_name: String,
    },

    /// This error indicates that the Helm wrapper failed to install the Helm
    /// release.
    #[snafu(display("failed to install Helm release {release_name}"))]
    InstallHelmRelease {
        release_name: String,
        source: helm::Error,
    },

    /// This error indicates that the Helm wrapper failed to uninstall the Helm
    /// release.
    #[snafu(display("failed to uninstall Helm chart"))]
    UninstallHelmRelease {
        release_name: String,
        source: helm::Error,
    },

    /// This error indicates that the Helm chart source kind is not supported.
    #[snafu(display("local Helm chart sources are not yet supported (source: {chart_source:?})"))]
    UnsupportedChartSource { chart_source: String },

    /// This error indicates that Helm chart options could not be serialized
    /// into YAML.
    #[snafu(display("failed to serialize Helm chart options"))]
    SerializeOptions { source: serde_yaml::Error },

    /// This error indicates that the creation of a kube client failed.
    #[snafu(display("failed to create Kubernetes client"))]
    CreateKubeClient { source: k8s::Error },

    /// This error indicates that the kube client failed to deloy manifests.
    #[snafu(display("failed to deploy manifests using the kube client"))]
    DeployManifest { source: k8s::Error },
}

pub trait InstallManifestsExt {
    // TODO (Techassi): This step shouldn't care about templating the manifests nor fetching them from remote
    #[instrument(skip_all, fields(%namespace, indicatif.pb_show = true))]
    #[allow(async_fn_in_trait)]
    async fn install_manifests(
        manifests: &[ManifestSpec],
        parameters: &HashMap<String, String>,
        namespace: &str,
        labels: Labels,
        client: &Client,
        transfer_client: &xfer::Client,
    ) -> Result<(), Error> {
        debug!("Installing manifests");

        Span::current().pb_set_style(&PROGRESS_BAR_STYLE);
        Span::current().pb_set_length(manifests.len() as u64);

        let mut parameters = parameters.clone();
        // We need some additional templating capabilities, e.g. the namespace, so that stacks/demos
        // can use that to render e.g. the fqdn service names [which contain the namespace].
        parameters.insert("NAMESPACE".to_owned(), namespace.to_owned());

        for manifest in manifests {
            let parameters = parameters.clone();

            match manifest {
                ManifestSpec::HelmChart(helm_file) => {
                    debug!(helm_file, "Installing manifest from Helm chart");

                    let helm_chart =
                        get_helm_chart(helm_file, transfer_client, &parameters).await?;

                    info!(helm_chart.name, helm_chart.version, "Installing Helm chart",);

                    let chart_source = match helm_chart.repo.source_kind() {
                        helm::ChartSourceKind::Repo => {
                            helm::add_repo(&helm_chart.repo.name, &helm_chart.repo.url).context(
                                AddHelmRepositorySnafu {
                                    repo_name: helm_chart.repo.name.clone(),
                                },
                            )?;
                            &helm_chart.repo.name
                        }
                        helm::ChartSourceKind::Oci => &helm_chart.repo.url,
                        helm::ChartSourceKind::Local => {
                            return UnsupportedChartSourceSnafu {
                                chart_source: helm_chart.repo.url.clone(),
                            }
                            .fail();
                        }
                    };

                    // Serialize chart options to string
                    let values_yaml = serde_yaml::to_string(&helm_chart.options)
                        .context(SerializeOptionsSnafu)?;

                    // Install the Helm chart using the Helm wrapper
                    helm::upgrade_or_install_release_from_repo_or_registry(
                        &helm_chart.release_name,
                        helm::ChartVersion {
                            chart_source,
                            chart_name: &helm_chart.name,
                            chart_version: Some(&helm_chart.version),
                        },
                        Some(&values_yaml),
                        namespace,
                        true,
                    )
                    .context(InstallHelmReleaseSnafu {
                        release_name: helm_chart.release_name,
                    })?;
                }
                ManifestSpec::PlainYaml(manifest_file) => {
                    debug!(manifest_file, "Installing YAML manifest");

                    // Read YAML manifest and apply templating
                    let path_or_url =
                        manifest_file
                            .into_path_or_url()
                            .context(ParsePathOrUrlSnafu {
                                path_or_url: manifest_file.clone(),
                            })?;

                    let manifests = transfer_client
                        .get(&path_or_url, &Template::new(&parameters))
                        .await
                        .context(FileTransferSnafu)?;

                    client
                        .deploy_manifests(&manifests, namespace, labels.clone())
                        .await
                        .context(DeployManifestSnafu)?;
                }
            }

            Span::current().pb_inc(1);
        }

        Ok(())
    }

    /// This function only handles uninstalling Helm Charts
    ///
    /// To delete objects installed through other manifests use [`Client::delete_namespace`] or [`Client::delete_all_objects_with_label`] instead.
    #[instrument(skip_all, fields(%namespace, indicatif.pb_show = true))]
    #[allow(async_fn_in_trait)]
    async fn uninstall_helm_manifests(
        manifests: &[ManifestSpec],
        parameters: &mut HashMap<String, String>,
        namespace: &str,
        transfer_client: &xfer::Client,
    ) -> Result<(), Error> {
        debug!("Uninstalling Helm manifests");
        Span::current().pb_set_message("Uninstalling Helm charts");

        // We add the NAMESPACE parameter, so that stacks/demos can use that to render e.g. the
        // fqdn service names [which contain the namespace].
        parameters.insert("NAMESPACE".to_owned(), namespace.to_owned());

        for manifest in manifests {
            match manifest {
                ManifestSpec::HelmChart(helm_file) => {
                    debug!(helm_file, "Uninstalling manifest from Helm chart");

                    let helm_chart = get_helm_chart(helm_file, transfer_client, parameters).await?;

                    info!(
                        helm_chart.name,
                        helm_chart.version, "Uninstalling Helm chart",
                    );

                    helm::uninstall_release(&helm_chart.release_name, namespace, true).context(
                        UninstallHelmReleaseSnafu {
                            release_name: &helm_chart.release_name,
                        },
                    )?;
                }
                ManifestSpec::PlainYaml(_) => {
                    // This function only handles uninstalling Helm Charts
                }
            }
        }

        Ok(())
    }
}

pub async fn get_helm_chart(
    helm_file: &str,
    transfer_client: &xfer::Client,
    parameters: &HashMap<String, String>,
) -> Result<helm::Chart, Error> {
    // Read Helm chart YAML and apply templating
    let helm_file_location = helm_file.into_path_or_url().context(ParsePathOrUrlSnafu {
        path_or_url: helm_file,
    })?;

    let helmchart = transfer_client
        .get(
            &helm_file_location,
            &Template::new(parameters).then(Yaml::default()),
        )
        .await
        .context(FileTransferSnafu)?;

    Ok(helmchart)
}
