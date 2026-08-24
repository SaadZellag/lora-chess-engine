fn main() {
    use std::env;
    // 1. Read the env var or fall back to a default path
    let file_path = env::var("EVALFILE").unwrap_or_else(|_| "default_nnue.rs".to_string());

    // 2. Pass the resolved path to your Rust code as a new env var
    println!("cargo:rustc-env=NNUE_PATH={}", file_path);

    // 3. Ensure Cargo re-runs this script if the env var or the file changes
    println!("cargo:rerun-if-env-changed=EVALFILE");
    println!("cargo:rerun-if-changed={}", file_path);
}