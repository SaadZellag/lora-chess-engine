use anyhow::{Result, anyhow};
use std::process::Command;
use std::path::{Path, PathBuf};
use log::info;
use crate::models::Settings;

pub async fn compile(
    ref_name: &str,
    settings: &Settings,
    env_vars: &serde_json::Value,
) -> Result<String> {
    // Check if binary already exists
    let cached_bin = get_binary_path(&settings.compiled_engines_path, ref_name);
    if cached_bin.exists() {
        info!("Using cached binary: {}", cached_bin.display());
        return Ok(cached_bin.to_string_lossy().to_string());
    }

    // Determine if ref_name is a branch or commit
    let _is_commit = ref_name.len() == 40 && ref_name.chars().all(|c| c.is_ascii_hexdigit());
    
    // Clone/fetch the repository
    setup_repo(&settings.lora_repo_path, ref_name, _is_commit).await?;

    // Compile with cargo
    let env_map = if let Some(obj) = env_vars.as_object() {
        obj.iter()
            .map(|(k, v)| (k.clone(), v.as_str().unwrap_or("").to_string()))
            .collect::<Vec<_>>()
    } else {
        vec![]
    };

    info!("Compiling with ref: {}", ref_name);
    
    let mut cmd = Command::new("cargo");
    cmd.current_dir(&settings.lora_repo_path);
    cmd.arg("build");
    cmd.arg("--release");
    cmd.arg("--bin");
    cmd.arg("lora");

    // Add environment variables
    for (key, val) in &env_map {
        cmd.env(key, val);
    }

    let output = cmd.output()?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("Compilation failed:\n{}", stderr));
    }

    // Copy binary to cache
    let source_bin = PathBuf::from(&settings.lora_repo_path)
        .join("target/release/lora");
    
    if !source_bin.exists() {
        return Err(anyhow!("Compiled binary not found at {}", source_bin.display()));
    }

    std::fs::create_dir_all(&settings.compiled_engines_path)?;
    std::fs::copy(&source_bin, &cached_bin)?;

    Ok(cached_bin.to_string_lossy().to_string())
}

async fn setup_repo(repo_path: &str, ref_name: &str, _is_commit: bool) -> Result<()> {
    let repo_path = Path::new(repo_path);

    if !repo_path.exists() {
        return Err(anyhow!("Repository path {} does not exist", repo_path.display()));
    }

    // Fetch latest changes
    let mut fetch_cmd = Command::new("git");
    fetch_cmd.current_dir(repo_path);
    fetch_cmd.arg("fetch");
    fetch_cmd.arg("origin");

    let fetch_output = fetch_cmd.output()?;
    if !fetch_output.status.success() {
        let stderr = String::from_utf8_lossy(&fetch_output.stderr);
        return Err(anyhow!("Git fetch failed:\n{}", stderr));
    }

    // Checkout the ref
    let mut checkout_cmd = Command::new("git");
    checkout_cmd.current_dir(repo_path);
    checkout_cmd.arg("checkout");
    checkout_cmd.arg(ref_name);

    let checkout_output = checkout_cmd.output()?;
    if !checkout_output.status.success() {
        let stderr = String::from_utf8_lossy(&checkout_output.stderr);
        return Err(anyhow!("Git checkout failed:\n{}", stderr));
    }

    Ok(())
}

fn get_binary_path(cache_dir: &str, ref_name: &str) -> PathBuf {
    let safe_name = ref_name.replace("/", "_").replace("\\", "_");
    PathBuf::from(cache_dir).join(format!("lora_{}", safe_name))
}
