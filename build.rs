use gobuild::BuildMode;

fn main() {
    gobuild::Build::new()
        .file("go-helm-wrapper/main.go")
        .buildmode(BuildMode::CArchive)
        .compile("go-helm-wrapper");
}
