use log::trace;
use std::process::{Command, Stdio};
use std::str;
use which::which;

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

pub fn execute_command_and_return_exit_code(mut args: Vec<&str>) -> i32 {
    let args_string = args.join(" ");
    trace!("Executing command \"{args_string}\"");

    let command = args.remove(0);
    Command::new(command)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .expect("")
        .code()
        .expect("")
}
