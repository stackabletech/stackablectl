use clap::Parser;
use comfy_table::{presets::UTF8_FULL, Cell, ContentArrangement, Table};
use indexmap::IndexMap;
use k8s_openapi::api::core::v1::Secret;
use kube::Api;
use serde::Serialize;
use std::{error::Error, vec};

use crate::{arguments::OutputType, kube::get_client};

use minio::get_minio_services;
use opensearch::get_opensearch_dashboards_services;

use self::stackable::get_stackable_services;

pub mod minio;
pub mod opensearch;
pub mod stackable;

pub static REDACTED_PASSWORD: &str = "<redacted>";

#[derive(Parser)]
pub enum CliCommandServices {
    /// List deployed services
    #[command(alias("ls"))]
    List {
        /// If specified services of all namespaces will be shown, not only the namespace you're currently in
        #[arg(short, long)]
        all_namespaces: bool,

        /// Don't show credentials in the output
        #[arg(short, long)]
        redact_credentials: bool,

        /// Show the product versions in the output
        #[arg(long)]
        show_versions: bool,

        #[arg(short, long, value_enum, default_value = "text")]
        output: OutputType,
    },
}

impl CliCommandServices {
    pub async fn handle(&self) -> Result<(), Box<dyn Error>> {
        match self {
            CliCommandServices::List {
                all_namespaces,
                output,
                redact_credentials,
                show_versions,
            } => {
                list_services(*all_namespaces, *redact_credentials, *show_versions, output).await?;
            }
        }

        Ok(())
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledProduct {
    pub name: String,
    pub namespace: Option<String>, // Some CRDs are cluster scoped
    pub endpoints: IndexMap<String, String>, // key: service name (e.g. web-ui), value: url
    pub extra_infos: Vec<String>,
}

async fn list_services(
    all_namespaces: bool,
    redact_credentials: bool,
    show_versions: bool,
    output_type: &OutputType,
) -> Result<(), Box<dyn Error>> {
    let mut output =
        get_stackable_services(!all_namespaces, redact_credentials, show_versions).await?;

    let minio = get_minio_services(!all_namespaces, redact_credentials).await?;
    if !minio.is_empty() {
        output.insert("minio".to_string(), minio);
    }

    let opensearch =
        get_opensearch_dashboards_services(!all_namespaces, redact_credentials).await?;
    if !opensearch.is_empty() {
        output.insert("opensearch-dashboards".to_string(), opensearch);
    }

    match output_type {
        OutputType::Text => {
            let mut table = Table::new();
            table
                .load_preset(UTF8_FULL)
                .set_content_arrangement(ContentArrangement::Dynamic)
                .set_header(vec![
                    Cell::new("Product"),
                    Cell::new("Name"),
                    Cell::new("Namespace"),
                    Cell::new("Endpoints"),
                    Cell::new("Info"),
                ]);

            let max_endpoint_name_length = output
                .values()
                .flatten()
                .flat_map(|p| &p.endpoints)
                .map(|e| e.0.len())
                .max()
                .unwrap_or_default();

            for (product_name, installed_products) in output {
                for installed_product in installed_products {
                    let mut endpoints = vec![];
                    for endpoint in &installed_product.endpoints {
                        endpoints.push(vec![endpoint.0.as_str(), endpoint.1.as_str()]);
                    }

                    let endpoints = installed_product
                        .endpoints
                        .iter()
                        .map(|(name, url)| {
                            format!("{name:width$}{url}", width = max_endpoint_name_length + 1)
                        })
                        .collect::<Vec<_>>()
                        .join("\n");

                    table.add_row(vec![
                        Cell::new(product_name.as_str()),
                        Cell::new(installed_product.name),
                        Cell::new(installed_product.namespace.unwrap_or_default()),
                        Cell::new(endpoints),
                        Cell::new(installed_product.extra_infos.join("\n")),
                    ]);
                }
            }

            println!("{table}");
        }
        OutputType::Json => {
            println!("{}", serde_json::to_string_pretty(&output)?);
        }
        OutputType::Yaml => {
            println!("{}", serde_yaml::to_string(&output)?);
        }
    }

    Ok(())
}

async fn get_credentials_from_secret(
    secret_name: &str,
    secret_namespace: &str,
    username_key: &str,
    password_key: &str,
    redact_credentials: bool,
) -> Result<Option<(String, String)>, Box<dyn Error>> {
    let client = get_client().await?;
    let secret_api: Api<Secret> = Api::namespaced(client, secret_namespace);

    let secret = secret_api.get(secret_name).await?;
    let secret_data = secret
        .data
        .ok_or(format!("Secret {secret_name} had no data"))?;

    match (secret_data.get(username_key), secret_data.get(password_key)) {
        (Some(username), Some(password)) => {
            let username = String::from_utf8(username.0.clone())?;
            let password = if redact_credentials {
                REDACTED_PASSWORD.to_string()
            } else {
                String::from_utf8(password.0.clone())?
            };
            Ok(Some((username, password)))
        }
        _ => Ok(None),
    }
}
