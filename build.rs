use std::{env, process::Command};

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    Command::new("sh")
        .arg("-c")
        .arg(format!("cd go-helm-wrapper/ && go build -buildmode=c-archive -ldflags -w -o {out_dir}/libhelm.a main.go && cd .."))
        .status().unwrap();

    println!("cargo:rustc-link-search=native={}", out_dir);
    println!("cargo:rustc-link-lib=static=helm");

    println!("cargo:rerun-if-changed=go-helm-wrapper/main.go");
}
