use log::trace;
use std::io::Write;
use std::process::{Command, Stdio};
use std::str;
use which::which;

#[repr(C)]
pub struct GoString {
    p: *const u8,
    n: i64,
}

impl From<&str> for GoString {
    fn from(str: &str) -> Self {
        GoString {
            p: str.as_ptr(),
            n: str.len() as i64,
        }
    }
}

/// Ensures that the program is installed
/// If the program is not installed it will panic
pub fn ensure_program_installed(program: &str) {
    which(program)
        .unwrap_or_else(|_| panic!("Could not find a installation of {program}. Please have a look at the README of stackablectl on what the prerequisites are: https://github.com/stackabletech/stackablectl"));
}

pub fn execute_command(mut args: Vec<&str>) -> String {
    assert!(!args.is_empty());

    let args_string = args.join(" ");
    trace!("Executing command \"{args_string}\"");

    let command = args.remove(0);
    let output = Command::new(command)
        .args(args)
        .output()
        .unwrap_or_else(|_| panic!("Failed to get output of the command \"{args_string}\""));

    if !output.status.success() {
        panic!(
            "Failed to execute the command \"{args_string}\". Stderr was: {}",
            str::from_utf8(&output.stderr).expect("Could not parse command stderr as utf-8")
        );
    }

    let stdout_string =
        str::from_utf8(&output.stdout).expect("Could not parse command response as utf-8");

    trace!("Command output for \"{args_string}\":\n{stdout_string}");

    stdout_string.to_string()
}

pub fn execute_command_with_stdin(mut args: Vec<&str>, stdin: &str) {
    assert!(!args.is_empty());

    let args_string = args.join(" ");
    trace!("Executing command \"{args_string}\" with the following stdin input:\n{stdin}");

    let command = args.remove(0);
    let child = Command::new(command)
        .args(args)
        .stdin(Stdio::piped())
        .spawn()
        .unwrap_or_else(|_| panic!("Failed to spawn the command \"{args_string}\""));

    child
        .stdin
        .as_ref()
        .unwrap()
        .write_all(stdin.as_bytes())
        .expect("Failed to write kind cluster definition via stdin");

    if !child.wait_with_output().unwrap().status.success() {
        panic!("Failed to execute the command \"{args_string}\"");
    }
}
