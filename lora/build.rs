// build.rs
use std::process::Command;

fn main() {
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .expect("failed to run git");

    let git_hash = String::from_utf8(output.stdout).unwrap();
    println!("cargo:rustc-env=GIT_HASH={}", git_hash.trim());

    // Optional: rerun build script if HEAD changes, so the hash stays fresh
    println!("cargo:rerun-if-changed=.git/HEAD");
}