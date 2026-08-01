use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Test {
    pub id: String,
    pub engine1_ref: String,      // branch or commit hash
    pub engine1_name: String,
    pub engine2_ref: String,      // branch or commit hash
    pub engine2_name: String,
    pub env_vars: serde_json::Value,  // compiler env vars
    pub fastchess_params: serde_json::Value,
    pub status: TestStatus,
    pub created_at: String,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
    pub games_played: i32,
    pub games_total: i32,
    pub discord_webhook: Option<String>,
    pub engine1_bin: Option<String>,
    pub engine2_bin: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TestStatus {
    Pending,
    Compiling,
    Running,
    Paused,
    Finished,
    Failed,
    Discarded,
}

impl TestStatus {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Pending => "pending",
            Self::Compiling => "compiling",
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Finished => "finished",
            Self::Failed => "failed",
            Self::Discarded => "discarded",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "pending" => Some(Self::Pending),
            "compiling" => Some(Self::Compiling),
            "running" => Some(Self::Running),
            "paused" => Some(Self::Paused),
            "finished" => Some(Self::Finished),
            "failed" => Some(Self::Failed),
            "discarded" => Some(Self::Discarded),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub default_env_vars: serde_json::Value,
    pub default_fastchess_params: serde_json::Value,
    pub compiled_engines_path: String,
    pub lora_repo_path: String,
    pub fastchess_path: String,
    pub default_discord_webhook: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestLog {
    pub id: String,
    pub test_id: String,
    pub timestamp: String,
    pub level: String,  // "info", "warn", "error"
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTestRequest {
    pub engine1_ref: String,
    pub engine1_name: String,
    pub engine2_ref: String,
    pub engine2_name: String,
    pub env_vars: Option<serde_json::Value>,
    pub fastchess_params: serde_json::Value,
    pub discord_webhook: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettingsRequest {
    pub default_env_vars: Option<serde_json::Value>,
    pub default_fastchess_params: Option<serde_json::Value>,
    pub compiled_engines_path: Option<String>,
    pub lora_repo_path: Option<String>,
    pub fastchess_path: Option<String>,
    pub default_discord_webhook: Option<String>,
}
