fn main() {
    // Provide a bundled `protoc` for dependencies that expect it (e.g. lancedb/lance).
    if let Ok(path) = protoc_bin_vendored::protoc_bin_path() {
        println!("cargo:warning=Using vendored protoc at {}", path.display());
        std::env::set_var("PROTOC", path);
    }
    println!("cargo:rerun-if-changed=build.rs");
}
