use crate::arguments::OutputType;
use clap::Parser;
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Stacks {
    stacks: IndexMap<String, Stack>,
}

#[derive(Debug, Deserialize, Serialize)]
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

fn get_stacks() -> Stacks {
    // TODO Read from URL/file/whatever
    let file_name = "stacks.yaml";
    let file = std::fs::File::open(file_name)
        .expect(format!("Could not read releases from {file_name}").as_str());

    serde_yaml::from_reader(file)
        .expect(format!("Failed to parse release list from {file_name}").as_str())
}
