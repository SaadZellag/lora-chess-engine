use anyhow::{Result, anyhow};
use std::process::{Command, Stdio};
use std::io::BufRead;
use log::info;
use crate::models::{Test, Settings};

pub async fn run_test(
    test_id: String,
    test: Test,
    settings: Settings,
) -> Result<()> {
    info!("Running fastchess test: {}", test_id);

    let engine1_bin = test.engine1_bin.ok_or_else(|| anyhow!("Engine 1 binary not set"))?;
    let engine2_bin = test.engine2_bin.ok_or_else(|| anyhow!("Engine 2 binary not set"))?;

    // Build fastchess command
    let mut cmd = Command::new(&settings.fastchess_path);
    
    // Add engine configurations
    cmd.arg("-engine");
    cmd.arg(format!("cmd={}", engine1_bin));
    cmd.arg(format!("name={}", test.engine1_name));

    cmd.arg("-engine");
    cmd.arg(format!("cmd={}", engine2_bin));
    cmd.arg(format!("name={}", test.engine2_name));

    // Add fastchess parameters from JSON
    if let Some(params_obj) = test.fastchess_params.as_object() {
        for (key, val) in params_obj {
            match val {
                serde_json::Value::String(s) => {
                    cmd.arg(format!("-{}", key));
                    cmd.arg(s);
                }
                serde_json::Value::Number(n) => {
                    cmd.arg(format!("-{}", key));
                    cmd.arg(n.to_string());
                }
                serde_json::Value::Bool(b) => {
                    if *b {
                        cmd.arg(format!("-{}", key));
                    }
                }
                serde_json::Value::Array(arr) => {
                    // Handle array parameters
                    for item in arr {
                        if let serde_json::Value::String(s) = item {
                            cmd.arg(s);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    info!("Running command: fastchess with parameters");
    
    let mut child = cmd.spawn()?;
    
    let stdout = child.stdout.take().ok_or_else(|| anyhow!("Failed to capture stdout"))?;
    let stderr = child.stderr.take().ok_or_else(|| anyhow!("Failed to capture stderr"))?;

    let test_id_clone = test_id.clone();
    
    // Log stdout in a separate task
    let test_id_stdout = test_id_clone.clone();
    tokio::spawn(async move {
        let reader = std::io::BufReader::new(stdout);
        for line in reader.lines() {
            if let Ok(line) = line {
                info!("[{}] {}", test_id_stdout, line);
            }
        }
    });

    // Log stderr in a separate task
    let test_id_stderr = test_id_clone.clone();
    tokio::spawn(async move {
        let reader = std::io::BufReader::new(stderr);
        for line in reader.lines() {
            if let Ok(line) = line {
                info!("[{}] ERROR: {}", test_id_stderr, line);
            }
        }
    });

    // Wait for completion
    let status = child.wait()?;

    if !status.success() {
        return Err(anyhow!("Fastchess exited with code: {:?}", status.code()));
    }

    info!("Test {} completed successfully", test_id);
    Ok(())
}
