fn main() {
    // Provide a bundled `protoc` for dependencies that expect it (e.g. lancedb/lance).
    if let Ok(path) = protoc_bin_vendored::protoc_bin_path() {
        println!("cargo:warning=Using vendored protoc at {}", path.display());
        // Propagate to dependent build scripts.
        println!("cargo:rustc-env=PROTOC={}", path.display());
    }
    println!("cargo:rerun-if-changed=build.rs");
}
