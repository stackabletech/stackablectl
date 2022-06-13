use std::error::Error;

use clap::Parser;
use indexmap::IndexMap;
use serde::Serialize;

use crate::{arguments::OutputType, kube};

#[derive(Parser)]
pub enum CliCommandServices {
    /// List deployed services
    #[clap(alias("ls"))]
    List {
        /// If specified services of all namespaces will be shown, not only the namespace you're currently in
        #[clap(short, long)]
        all_namespaces: bool,

        #[clap(short, long, arg_enum, default_value = "text")]
        output: OutputType,
    },
}

impl CliCommandServices {
    pub async fn handle(&self) -> Result<(), Box<dyn Error>> {
        match self {
            CliCommandServices::List {
                all_namespaces,
                output,
            } => list_services(*all_namespaces, output).await?,
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
}

async fn list_services(
    all_namespaces: bool,
    output_type: &OutputType,
) -> Result<(), Box<dyn Error>> {
    let output = kube::get_services(!all_namespaces).await?;

    match output_type {
        OutputType::Text => {
            println!("PRODUCT              NAMESPACE                      NAME                                     ENDPOINTS");
            for (product_name, installed_products) in output.iter() {
                for installed_product in installed_products {
                    println!(
                        "{:20} {:30} {:40} {}",
                        product_name,
                        installed_product
                            .namespace
                            .as_ref()
                            .unwrap_or(&"~".to_string()),
                        installed_product.name,
                        installed_product.endpoints.iter().map(|(name, url)| {
                            format!("{:10} {url}", format!("{name}:"))
                        }).collect::<Vec<_>>().join("\n                                                                                             ")
                    );
                }
            }
        }
        OutputType::Json => {
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        OutputType::Yaml => {
            println!("{}", serde_yaml::to_string(&output).unwrap());
        }
    }

    Ok(())
}
