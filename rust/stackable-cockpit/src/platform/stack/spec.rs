use serde::{Deserialize, Serialize};
use yaml_serde::Mapping;
use snafu::{OptionExt, ResultExt, Snafu};
use stackable_operator::kvp::{Label, LabelError};
use tracing::{Span, debug, info, instrument, log::warn};
use tracing_indicatif::span_ext::IndicatifSpanExt as _;
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

use crate::{
    common::manifest::ManifestSpec,
    platform::{
        cluster::{ResourceRequests, ResourceRequestsError},
        manifests::{self, InstallManifestsExt},
        namespace,
        operator::ChartSourceType,
        release,
        stack::{StackInstallParameters, StackUninstallParameters},
    },
    utils::{
        k8s::{self, Client},
        params::{
            IntoParameters, IntoParametersError, Parameter, RawParameter, RawParameterParseError,
        },
    },
    xfer,
};

pub type RawStackParameterParseError = RawParameterParseError;
pub type RawStackParameter = RawParameter;
pub type StackParameter = Parameter;

#[derive(Debug, Snafu)]
pub enum Error {
    /// This error indicates that parsing a string into stack / demo parameters
    /// failed.
    #[snafu(display("failed to parse demo / stack parameters"))]
    ParseParameters { source: IntoParametersError },

    /// This error indicates that the requested release doesn't exist in the
    /// loaded list of releases.
    #[snafu(display("no release named {name:?}"))]
    NoSuchRelease { name: String },

    /// This error indicates that the release failed to install.
    #[snafu(display("failed to install release"))]
    InstallRelease { source: release::Error },

    /// This error indicates that the release failed to uninstall.
    #[snafu(display("failed to uninstall release"))]
    UninstallRelease { source: release::Error },

    #[snafu(display("stack resource requests error"), context(false))]
    StackResourceRequests { source: ResourceRequestsError },

    /// This error indicates that the stack doesn't support being installed in
    /// the provided namespace.
    #[snafu(display("unable install stack in namespace {requested:?}, only {supported:?} supported", supported = supported.join(", ")))]
    UnsupportedNamespace {
        requested: String,
        supported: Vec<String>,
    },

    #[snafu(display("failed to create namespace {namespace:?}"))]
    CreateNamespace {
        source: namespace::Error,
        namespace: String,
    },

    #[snafu(display("failed to install stack manifests"))]
    InstallManifests { source: manifests::Error },

    #[snafu(display("failed to uninstall Helm manifests"))]
    UninstallHelmManifests { source: manifests::Error },

    #[snafu(display("failed to delete object"))]
    DeleteObject { source: k8s::Error },

    #[snafu(display("failed to build label"))]
    BuildLabel { source: LabelError },
}

/// This struct describes a stack with the v2 spec
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct StackSpec {
    /// A short description of the stack
    pub description: String,

    /// The release used by the stack, e.g. 23.4
    #[serde(rename = "stackableRelease")]
    pub release: String,

    /// Supported namespaces this stack can run in. An empty list indicates that
    /// the stack can run in any namespace.
    #[serde(default)]
    pub supported_namespaces: Vec<String>,

    /// A variable number of operators
    #[serde(rename = "stackableOperators")]
    pub operators: Vec<String>,

    /// A variable number of labels (tags)
    #[serde(default)]
    pub labels: Vec<String>,

    /// A variable number of Helm or YAML manifests
    #[serde(default)]
    pub manifests: Vec<ManifestSpec>,

    /// The resource requests the stack imposes on a Kubernetes cluster
    pub resource_requests: Option<ResourceRequests>,

    /// A variable number of supported parameters
    #[serde(default)]
    pub parameters: Vec<StackParameter>,
}

impl InstallManifestsExt for StackSpec {}

impl StackSpec {
    /// Checks if the prerequisites to run this stack are met. These checks
    /// include:
    ///
    /// - Does the stack support to be installed in the requested namespace?
    /// - Does the cluster have enough resources available to run this stack?
    #[instrument(skip_all)]
    pub async fn check_prerequisites(&self, client: &Client, namespace: &str) -> Result<(), Error> {
        debug!("Checking prerequisites before installing stack");

        // Returns an error if the stack doesn't support to be installed in the
        // requested product namespace. When installing a demo, this check is
        // already done on the demo spec level, however we still need to check
        // here, as stacks can be installed on their own.
        if !self.supports_namespace(namespace) {
            return Err(Error::UnsupportedNamespace {
                supported: self.supported_namespaces.clone(),
                requested: namespace.to_owned(),
            });
        }

        // Checks if the available cluster resources are sufficient to deploy
        // the demo.
        if let Some(resource_requests) = &self.resource_requests {
            if let Err(err) = resource_requests
                .validate_cluster_size(client, "stack")
                .await
            {
                match err {
                    ResourceRequestsError::ValidationErrors { errors } => {
                        for error in errors {
                            warn!("{error}");
                        }
                    }
                    err => return Err(err.into()),
                }
            }
        }

        Ok(())
    }

    // TODO (Techassi): Can we get rid of the release list and just use the release spec instead
    #[instrument(skip_all, fields(
        stack_name = %install_parameters.stack_name,
        stack_namespace = %install_parameters.stack_namespace,
    ))]
    pub async fn install(
        &self,
        release_list: release::ReleaseList,
        install_parameters: StackInstallParameters,
        client: &Client,
        transfer_client: &xfer::Client,
    ) -> Result<(), Error> {
        // First, we check if the prerequisites are met
        self.check_prerequisites(client, &install_parameters.stack_namespace)
            .await?;

        // Second, we install the release if not opted out
        if !install_parameters.skip_release {
            namespace::create_if_needed(client, install_parameters.operator_namespace.clone())
                .await
                .context(CreateNamespaceSnafu {
                    namespace: install_parameters.operator_namespace.clone(),
                })?;

            self.install_release(
                release_list,
                &install_parameters.operator_namespace,
                &install_parameters.chart_source,
                &install_parameters.operator_values,
            )
            .await?;
        }

        // Next, create the product namespace if needed
        // TODO (@NickLarsenNZ): Remove clones (update create_if_needed to take a &str)
        namespace::create_if_needed(client, install_parameters.stack_namespace.clone())
            .await
            .context(CreateNamespaceSnafu {
                namespace: install_parameters.stack_namespace.clone(),
            })?;

        // Finally install the stack manifests
        self.prepare_manifests(install_parameters, client, transfer_client)
            .await
    }

    #[instrument(skip_all, fields(
        stack_name = %uninstall_parameters.stack_name,
        stack_namespace = %uninstall_parameters.stack_namespace,
    ))]
    pub async fn uninstall(
        &self,
        release_list: release::ReleaseList,
        uninstall_parameters: StackUninstallParameters,
        client: &Client,
        transfer_client: &xfer::Client,
    ) -> Result<(), Error> {
        // Uninstall Helm Charts
        let parameters = &mut Vec::new()
            .into_params(self.parameters.clone())
            .context(ParseParametersSnafu)?;

        // We add the STACK and optionally DEMO parameter, so that stacks can use that to render e.g. the stack label
        parameters.insert("STACK".to_owned(), uninstall_parameters.stack_name.clone());
        if let Some(demo_name) = uninstall_parameters.demo_name {
            parameters.insert("DEMO".to_owned(), demo_name);
        }

        Self::uninstall_helm_manifests(
            &self.manifests,
            parameters,
            &uninstall_parameters.stack_namespace.to_owned(),
            transfer_client,
        )
        .await
        .context(UninstallHelmManifestsSnafu)?;

        // Delete stack namespace
        client
            .delete_namespace(uninstall_parameters.stack_namespace)
            .await
            .context(DeleteObjectSnafu)?;

        // Delete remaining objects not namespace scoped
        client
            .delete_all_objects_with_label(
                Label::try_from(("stackable.tech/stack", &uninstall_parameters.stack_name))
                    .context(BuildLabelSnafu)?,
                None,
            )
            .await
            .context(DeleteObjectSnafu)?;

        // Delete operators and the operator namespace
        if !uninstall_parameters.skip_operators {
            self.uninstall_release(release_list, &uninstall_parameters.operator_namespace)
                .await?;

            client
                .delete_namespace(uninstall_parameters.operator_namespace)
                .await
                .context(DeleteObjectSnafu)?;
        }

        // Delete CRDs
        if !uninstall_parameters.skip_crds {
            client
                .delete_crds_with_group_suffix("stackable.tech")
                .await
                .context(DeleteObjectSnafu)?;
        }

        Ok(())
    }

    #[instrument(skip_all, fields(release = %self.release, %operator_namespace, indicatif.pb_show = true))]
    pub async fn install_release(
        &self,
        release_list: release::ReleaseList,
        operator_namespace: &str,
        chart_source: &ChartSourceType,
        operator_values: &Mapping,
    ) -> Result<(), Error> {
        info!(self.release, "Trying to install release");
        Span::current().pb_set_message("Installing operators");

        // Get the release by name
        let release = release_list
            .get(&self.release)
            .context(NoSuchReleaseSnafu {
                name: self.release.clone(),
            })?;

        // Install the release
        release
            .install(
                &self.operators,
                &[],
                operator_namespace,
                chart_source,
                operator_values,
            )
            .await
            .context(InstallReleaseSnafu)
    }

    #[instrument(skip_all, fields(release = %self.release, %operator_namespace, indicatif.pb_show = true))]
    pub async fn uninstall_release(
        &self,
        release_list: release::ReleaseList,
        operator_namespace: &str,
    ) -> Result<(), Error> {
        info!(self.release, "Trying to uninstall release");
        Span::current().pb_set_message("Uninstalling operators");

        // Get the release by name
        let release = release_list
            .get(&self.release)
            .context(NoSuchReleaseSnafu {
                name: self.release.clone(),
            })?;

        // Uninstall the release
        release
            .uninstall(&self.operators, &[], operator_namespace)
            .context(UninstallReleaseSnafu)
    }

    #[instrument(skip_all, fields(indicatif.pb_show = true))]
    pub async fn prepare_manifests(
        &self,
        install_parameters: StackInstallParameters,
        client: &Client,
        transfer_client: &xfer::Client,
    ) -> Result<(), Error> {
        info!("Installing stack manifests");
        Span::current().pb_set_message("Installing manifests");

        let mut parameters = install_parameters
            .parameters
            .to_owned()
            .into_params(&self.parameters)
            .context(ParseParametersSnafu)?;

        // We add the STACK and optionally DEMO parameter, so that stacks can use that to render e.g. the stack label
        parameters.insert("STACK".to_owned(), install_parameters.stack_name);
        if let Some(demo_name) = install_parameters.demo_name {
            parameters.insert("DEMO".to_owned(), demo_name);
        }

        Self::install_manifests(
            &self.manifests,
            &parameters,
            &install_parameters.stack_namespace,
            install_parameters.labels,
            client,
            transfer_client,
        )
        .await
        .context(InstallManifestsSnafu)
    }

    fn supports_namespace(&self, namespace: impl Into<String>) -> bool {
        self.supported_namespaces.is_empty()
            || self.supported_namespaces.contains(&namespace.into())
    }
}
