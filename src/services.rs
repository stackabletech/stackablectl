use std::error::Error;

use clap::Parser;

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

async fn list_services(
    all_namespaces: bool,
    output_type: &OutputType,
) -> Result<(), Box<dyn Error>> {
    let output = kube::get_services(!all_namespaces).await?;

    match output_type {
        OutputType::Text => {
            println!(
                "SERVICE                                  NAMESPACE                      PORTS"
            );
            for (service_name, service_entry) in output.iter() {
                println!(
                    "{:40} {:30} {}",
                    service_name,
                    service_entry.namespace,
                    service_entry
                        .ports
                        .iter()
                        .filter_map(|port| port.name.clone())
                        .collect::<Vec<_>>()
                        .join(", ")
                );
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
