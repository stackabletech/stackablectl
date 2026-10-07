use futures::{StreamExt as _, TryStreamExt};
use indexmap::IndexMap;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_yaml::Mapping;
use snafu::{ResultExt, Snafu};
use tokio::task::JoinError;
use tracing::{Instrument, Span, debug, info, instrument};
use tracing_indicatif::span_ext::IndicatifSpanExt as _;
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

use crate::{
    PROGRESS_BAR_STYLE, helm,
    platform::{
        operator::{self, ChartSourceType, OperatorSpec},
        product,
    },
    utils::{
        k8s::{self, Client},
        path::{IntoPathOrUrl as _, PathOrUrlParseError},
        yaml::values_for_operator,
    },
    xfer::{self, processor::Text},
};

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("failed to parse operator spec"))]
    OperatorSpecParse { source: operator::SpecParseError },

    /// This error indicates that parsing a string into a path or URL failed.
    #[snafu(display("failed to parse {path_or_url:?} as path/url"))]
    ParsePathOrUrl {
        source: PathOrUrlParseError,
        path_or_url: String,
    },

    /// This error indicates that receiving remote content failed.
    #[snafu(display("failed to receive remote content"))]
    FileTransfer { source: xfer::Error },

    #[snafu(display("failed to install release using Helm"))]
    HelmInstall { source: helm::Error },

    #[snafu(display("failed to uninstall release using Helm"))]
    HelmUninstall { source: helm::Error },

    #[snafu(display("failed to launch background task"))]
    BackgroundTask { source: JoinError },

    #[snafu(display("failed to deploy manifests using the kube client"))]
    DeployManifest {
        #[snafu(source(from(k8s::Error, Box::new)))]
        source: Box<k8s::Error>,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct ReleaseSpec {
    /// Date this released was released
    #[serde(rename = "releaseDate")]
    pub date: String,

    /// A short description of this release
    pub description: String,

    /// List of products and their version in this release
    pub products: IndexMap<String, product::ProductSpec>,
}

impl ReleaseSpec {
    /// Installs a release by installing individual operators.
    #[instrument(skip_all, fields(
        %namespace,
        product.included = tracing::field::Empty,
        product.excluded = tracing::field::Empty,
        indicatif.pb_show = true
    ))]
    pub async fn install(
        &self,
        include_products: &[String],
        exclude_products: &[String],
        namespace: &str,
        chart_source: &ChartSourceType,
        operator_values: &Mapping,
    ) -> Result<()> {
        info!("Installing release");
        Span::current().pb_set_style(&PROGRESS_BAR_STYLE);

        include_products.iter().for_each(|product| {
            Span::current().record("product.included", product);
        });
        exclude_products.iter().for_each(|product| {
            Span::current().record("product.excluded", product);
        });

        let operators = self.filter_products(include_products, exclude_products);

        Span::current().pb_set_length(operators.len() as u64);

        let namespace = namespace.to_string();
        futures::stream::iter(operators)
            .map(|(product_name, product)| {
                let task_span =
                    tracing::info_span!("install_operator", product_name = tracing::field::Empty);

                let namespace = namespace.clone();
                let chart_source = chart_source.clone();
                let operator_helm_values = values_for_operator(operator_values, &product_name);
                // Helm installs currently `block_in_place`, so we need to spawn each job onto a separate task to
                // get useful parallelism.
                tokio::spawn(
                    async move {
                        Span::current().record("product_name", &product_name);
                        info!("Installing {product_name}-operator");

                        // Create operator spec
                        let operator =
                            OperatorSpec::new(&product_name, Some(product.version.clone()))
                                .context(OperatorSpecParseSnafu)?;

                        // Install operator
                        operator
                            .install(&namespace, &chart_source, &operator_helm_values)
                            .context(HelmInstallSnafu)?;

                        info!("Installed {product_name}-operator");

                        Ok(())
                    }
                    .instrument(task_span),
                )
            })
            .buffer_unordered(10)
            .map(|res| {
                Span::current().pb_inc(1);
                res.context(BackgroundTaskSnafu)?
            })
            .try_collect::<()>()
            .await
    }

    /// Upgrades a release by upgrading individual operators.
    #[instrument(skip_all, fields(
        %namespace,
        indicatif.pb_show = true
    ))]
    pub async fn upgrade_crds(
        &self,
        include_products: &[String],
        exclude_products: &[String],
        namespace: &str,
        k8s_client: &Client,
        transfer_client: &xfer::Client,
    ) -> Result<()> {
        info!("Upgrading CRDs for release");
        Span::current().pb_set_style(&PROGRESS_BAR_STYLE);

        include_products.iter().for_each(|product| {
            Span::current().record("product.included", product);
        });
        exclude_products.iter().for_each(|product| {
            Span::current().record("product.excluded", product);
        });

        let operators = self.filter_products(include_products, exclude_products);

        Span::current().pb_set_length(operators.len() as u64);

        for (product_name, product) in operators {
            info!("Upgrading CRDs for {product_name}-operator");
            let iter_span = tracing::info_span!("upgrade_crds_iter", indicatif.pb_show = true);

            async move {
                Span::current().pb_set_message(format!("Upgrading CRDs for {product_name}-operator").as_str());

                let release_branch = match product.version.pre.as_str() {
                    "dev" => "main".to_string(),
                    _ => {
                        product.version.to_string()
                    }
                };

                let request_url_string = &format!(
                    "https://raw.githubusercontent.com/stackabletech/{product_name}-operator/{release_branch}/deploy/helm/{product_name}-operator/crds/crds.yaml"
                );
                let request_url = request_url_string.into_path_or_url().context(ParsePathOrUrlSnafu {
                    path_or_url: request_url_string,
                })?;

                // Get CRD manifests from request_url
                let crd_manifests = transfer_client
                    .get(&request_url, &Text)
                    .await;
                let crd_manifests = match crd_manifests {
                        Ok(crd_manifests) => crd_manifests,
                        Err(crate::xfer::Error::FetchRemoteContent{source: reqwest_error})
                            if reqwest_error.status() == Some(StatusCode::NOT_FOUND) => {
                            // Ignore 404, as CRD versioning is rolled out to operators.
                            // Starting with secret-operator 25.11.0, the CRD is maintained by the operator,
                            // making this entire functionality obsolete.
                            // As only some of the operators are migrated yet, some operator crds.yaml's
                            // return a 404, others don't.
                            debug!(
                                product = product_name,
                                // https://opentelemetry.io/docs/specs/semconv/http/http-spans/#http-client-span
                                url.full = request_url_string,
                                "Skipped updating CRD, as it doesn't exist in the upstream GitHub repo because the operator most likely uses CRD versioning"
                            );
                            return Ok(());
                        },
                        Err(err) => {
                            return Err(Error::FileTransfer { source: err });
                        },
                    };

                // Upgrade CRDs
                k8s_client
                    .replace_crds(&crd_manifests)
                    .await
                    .context(DeployManifestSnafu)?;

                info!("Upgraded {product_name}-operator CRDs");

                Ok::<(), Error>(())
            }.instrument(iter_span).await?;

            Span::current().pb_inc(1);
        }

        Ok(())
    }

    #[instrument(skip_all, fields(indicatif.pb_show = true))]
    pub fn uninstall(
        &self,
        include_products: &[String],
        exclude_products: &[String],
        namespace: &str,
    ) -> Result<()> {
        info!("Uninstalling release");

        include_products.iter().for_each(|product| {
            Span::current().record("product.included", product);
        });
        exclude_products.iter().for_each(|product| {
            Span::current().record("product.excluded", product);
        });

        let operators = self.filter_products(include_products, exclude_products);

        Span::current().pb_set_style(&PROGRESS_BAR_STYLE);
        Span::current().pb_set_length(operators.len() as u64);

        for (product_name, product_spec) in operators {
            info!("Uninstalling {product_name}-operator");

            // Create operator spec
            let operator = OperatorSpec::new(product_name, Some(product_spec.version.clone()))
                .context(OperatorSpecParseSnafu)?;

            // Uninstall operator
            helm::uninstall_release(&operator.helm_name(), namespace, true)
                .context(HelmUninstallSnafu)?;

            Span::current().pb_inc(1);
        }

        Ok(())
    }

    /// Filters out products based on if they are included or excluded.
    pub fn filter_products(
        &self,
        include_products: &[String],
        exclude_products: &[String],
    ) -> Vec<(String, product::ProductSpec)> {
        self.products
            .iter()
            .filter(|(name, _)| include_products.is_empty() || include_products.contains(name))
            .filter(|(name, _)| !exclude_products.contains(name))
            .map(|(name, product)| (name.clone(), product.clone()))
            .collect()
    }
}
