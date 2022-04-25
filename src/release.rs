use crate::arguments::OutputType;
use clap::Parser;
use indexmap::IndexMap;
use log::error;
use serde::{Deserialize, Serialize};
use std::process::exit;

#[derive(Parser)]
pub enum CliCommandRelease {
    /// List all the available releases
    #[clap(alias("ls"))]
    List {
        #[clap(short, long, arg_enum, default_value = "text")]
        output: OutputType,
    },
    /// Show details of a specific release
    #[clap(alias("desc"))]
    Describe {
        release: String,
        #[clap(short, long, arg_enum, default_value = "text")]
        output: OutputType,
    },
}

impl CliCommandRelease {
    pub fn handle(&self) {
        match self {
            CliCommandRelease::List { output } => list_releases(output),
            CliCommandRelease::Describe { release, output } => describe_release(release, output),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Releases {
    releases: IndexMap<String, Release>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Release {
    release_date: String,
    description: String,
    operators: IndexMap<String, String>,
}

fn list_releases(output_type: &OutputType) {
    let output = get_releases();
    match output_type {
        OutputType::Text => {
            println!("RELEASE            RELEASE DATE   DESCRIPTION");
            for (release_name, release_entry) in output.releases.iter() {
                println!(
                    "{:18} {:14} {}",
                    release_name, release_entry.release_date, release_entry.description,
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
}

fn describe_release(release_name: &str, output_type: &OutputType) {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Output {
        release: String,
        release_date: String,
        description: String,
        operators: IndexMap<String, String>,
    }

    let releases = get_releases();
    let output = releases.releases
        .get(release_name)
        .map(|release| {
            Output {
                release: release_name.to_string(),
                release_date: release.release_date.to_string(),
                description: release.description.to_string(),
                operators: release.operators.clone(),
            }
        })
        .unwrap_or_else(|| {
        error!("Release {release_name} not found. Use `stackablectl release list` to list the available releases.");
        exit(1);
    });

    match output_type {
        OutputType::Text => {
            println!("Release:            {}", output.release);
            println!("Release date:       {}", output.release_date);
            println!("Description:        {}", output.description);
            println!(
                "Operators:          {}",
                output
                    .operators
                    .iter()
                    .map(|operator| format!("{}={}", operator.0, operator.1))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }
        OutputType::Json => {
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        OutputType::Yaml => {
            println!("{}", serde_yaml::to_string(&output).unwrap());
        }
    }
}

fn get_releases() -> Releases {
    // TODO Read from URL/file/whatever
    let file_name = "releases.yaml";
    let file = std::fs::File::open(file_name)
        .expect(format!("Could not read releases from {file_name}").as_str());

    serde_yaml::from_reader(file)
        .expect(format!("Failed to parse release list from {file_name}").as_str())
}
