use crate::arguments::OutputType;
use crate::{helpers, CliArgs};
use cached::proc_macro::cached;
use clap::Parser;
use indexmap::IndexMap;
use lazy_static::lazy_static;
use log::{error, warn};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::process::exit;
use std::sync::Mutex;

lazy_static! {
    pub static ref STACK_FILES: Mutex<Vec<String>> = Mutex::new(vec![
        "https://raw.githubusercontent.com/stackabletech/stackablectl/main/stacks.yaml".to_string(),
    ]);
}

#[derive(Parser)]
pub enum CliCommandStack {
    /// List all the available stack
    #[clap(alias("ls"))]
    List {
        #[clap(short, long, arg_enum, default_value = "text")]
        output: OutputType,
    },
    /// Show details of a specific stack
    #[clap(alias("desc"))]
    Describe {
        stack: String,

        #[clap(short, long, arg_enum, default_value = "text")]
        output: OutputType,
    },
}

impl CliCommandStack {
    pub fn handle(&self) {
        match self {
            CliCommandStack::List { output } => list_stacks(output),
            CliCommandStack::Describe { stack, output } => describe_stack(stack, output),
        }
    }
}

pub fn handle_common_cli_args(args: &CliArgs) {
    let mut stack_files = STACK_FILES.lock().unwrap();
    stack_files.append(&mut args.additional_stack_files.clone());
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Stacks {
    stacks: IndexMap<String, Stack>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Stack {
    description: String,
    stackable_release: String,
    labels: Vec<String>,
    manifests: String,
}

fn list_stacks(output_type: &OutputType) {
    let output = get_stacks();
    match output_type {
        OutputType::Text => {
            println!("STACK                               STACKABLE RELEASE  DESCRIPTION");
            for (stack_name, stack) in output.stacks.iter() {
                println!(
                    "{:35} {:18} {}",
                    stack_name, stack.stackable_release, stack.description,
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

fn describe_stack(stack_name: &str, output_type: &OutputType) {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Output {
        stack: String,
        description: String,
        stackable_release: String,
        labels: Vec<String>,
        manifests: String,
    }

    let stack = get_stack(stack_name);
    let output = Output {
        stack: stack_name.to_string(),
        description: stack.description,
        stackable_release: stack.stackable_release,
        labels: stack.labels,
        manifests: stack.manifests,
    };

    match output_type {
        OutputType::Text => {
            println!("Stack:              {}", output.stack);
            println!("Description:        {}", output.description);
            println!("Manifests:          {}", output.manifests);
            println!("Labels:             {}", output.labels.join(", "));
        }
        OutputType::Json => {
            println!("{}", serde_json::to_string_pretty(&output).unwrap());
        }
        OutputType::Yaml => {
            println!("{}", serde_yaml::to_string(&output).unwrap());
        }
    }
}

/// Cached because of potential slow network calls
#[cached]
fn get_stacks() -> Stacks {
    let mut all_stacks: IndexMap<String, Stack> = IndexMap::new();
    for stack_file in STACK_FILES.lock().unwrap().deref() {
        let yaml = helpers::read_from_url_or_file(stack_file);
        match yaml {
            Ok(yaml) => match serde_yaml::from_str::<Stacks>(&yaml) {
                Ok(stacks) => all_stacks.extend(stacks.stacks),
                Err(err) => warn!("Failed to parse stack list from {stack_file}: {err}"),
            },
            Err(err) => {
                warn!("Could not read from stacks file \"{stack_file}\": {err}");
            }
        }
    }

    Stacks { stacks: all_stacks }
}

fn get_stack(stack_name: &str) -> Stack {
    get_stacks()
        .stacks
        .remove(stack_name) // We need to remove to take ownership
        .unwrap_or_else(|| {
            error!("Stack {stack_name} not found. Use `stackablectl stack list` to list the available stacks.");
            exit(1);
        })
}
