use which::which;

pub fn ensure_program_installed(program: &str) {
    which(program)
        .unwrap_or_else(|_| panic!("Could not find a installation of {program}. Please have a look at the README of stackablectl on what the prerequisites are: https://github.com/stackabletech/stackablectl"));
}
