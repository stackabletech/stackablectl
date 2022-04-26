use crate::arguments::OutputType;
use crate::{helpers, CliArgs};
use cached::proc_macro::cached;
use clap::Parser;
use indexmap::IndexMap;
use lazy_static::lazy_static;
use log::warn;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use std::sync::Mutex;

lazy_static! {
    pub static ref STACK_FILES: Mutex<Vec<String>> = Mutex::new(vec!["stacks.yaml".to_string()]);
}

#[derive(Parser)]
pub enum CliCommandStack {
    /// List all the available stack
    #[clap(alias("ls"))]
    List {
        #[clap(short, long, arg_enum, default_value = "text")]
        output: OutputType,
    },
}

impl CliCommandStack {
    pub fn handle(&self) {
        match self {
            CliCommandStack::List { output } => list_stacks(output),
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

/// Cached because of potential slow network calls
#[cached]
fn get_stacks() -> Stacks {
    let mut all_stacks: IndexMap<String, Stack> = IndexMap::new();
    for stack_file in STACK_FILES.lock().unwrap().deref() {
        let yaml = helpers::read_from_url_or_file(&stack_file);
        match yaml {
            Ok(yaml) => {
                let stacks: Stacks = serde_yaml::from_str(&yaml)
                    .expect(format!("Failed to parse stack list from {stack_file}").as_str());
                all_stacks.extend(stacks.stacks.clone());
            }
            Err(err) => {
                warn!("Could not read from stacks file \"{stack_file}\": {err}");
            }
        }
    }

    Stacks { stacks: all_stacks }
}
