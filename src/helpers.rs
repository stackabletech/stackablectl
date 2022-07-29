use log::trace;
use std::{
    error::Error,
    ffi::CStr,
    fs,
    io::Write,
    os::raw::c_char,
    process::{Command, Stdio},
    str,
};
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

pub fn c_str_ptr_to_str(ptr: *const c_char) -> &'static str {
    let c_str = unsafe { CStr::from_ptr(ptr) };
    c_str.to_str().unwrap()
}

pub async fn read_from_url_or_file(url_or_file: &str) -> Result<String, String> {
    if let Ok(str) = fs::read_to_string(url_or_file) {
        return Ok(str);
    }

    match reqwest::get(url_or_file).await {
        Ok(response) => Ok(response.text().await.unwrap()),
        Err(err) => Err(format!(
            "Couldn't read a file or a URL with the name \"{url_or_file}\": {err}"
        )),
    }
}

/// Ensures that the program is installed
/// If the program is not installed it will return an Error
pub fn ensure_program_installed(program: &str) -> Result<(), Box<dyn Error>> {
    match which(program) {
        Ok(_) => Ok(()),
        Err(err) => Err(format!("Could not find a installation of {program}: {err}").into()),
    }
}

pub fn execute_command(mut args: Vec<&str>) -> Result<String, Box<dyn Error>> {
    assert!(!args.is_empty());

    let args_string = args.join(" ");
    trace!("Executing command \"{args_string}\"");

    let command = args.remove(0);
    let output = Command::new(command)
        .args(args)
        .output()
        .map_err(|err| format!("Failed to get output of the command \"{args_string}\": {err}"))?;

    if !output.status.success() {
        return Err(format!(
            "Failed to execute the command \"{args_string}\". Stderr was: {}",
            str::from_utf8(&output.stderr).expect("Could not parse command stderr as utf-8")
        )
        .into());
    }

    let stdout_string =
        str::from_utf8(&output.stdout).expect("Could not parse command response as utf-8");

    trace!("Command output for \"{args_string}\":\n{stdout_string}");

    Ok(stdout_string.to_string())
}

pub fn execute_command_with_stdin(mut args: Vec<&str>, stdin: &str) -> Result<(), Box<dyn Error>> {
    assert!(!args.is_empty());

    let args_string = args.join(" ");
    trace!("Executing command \"{args_string}\" with the following stdin input:\n{stdin}");

    let command = args.remove(0);
    let child = Command::new(command)
        .args(args)
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|err| format!("Failed to spawn the command \"{args_string}\": {err}"))?;

    child.stdin.as_ref().unwrap().write_all(stdin.as_bytes())?;

    if child.wait_with_output()?.status.success() {
        Ok(())
    } else {
        Err(format!("Failed to execute the command \"{args_string}\"").into())
    }
}
