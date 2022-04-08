fn main() {
    let path = "./go-helm-wrapper/";
    let lib = "helm";

    println!("cargo:rustc-link-search=native={}", path);
    println!("cargo:rustc-link-lib=static={}", lib);
}
