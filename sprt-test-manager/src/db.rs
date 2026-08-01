use rusqlite::{Connection, params, Result as SqlResult, Row};
use crate::models::{Test, TestStatus, Settings, TestLog};
use std::path::Path;
use log::info;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new<P: AsRef<Path>>(path: P) -> SqlResult<Self> {
        let conn = Connection::open(path)?;
        let db = Database { conn };
        db.init_schema()?;
        info!("Database initialized");
        Ok(db)
    }

    fn init_schema(&self) -> SqlResult<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS tests (
                id TEXT PRIMARY KEY,
                engine1_ref TEXT NOT NULL,
                engine1_name TEXT NOT NULL,
                engine2_ref TEXT NOT NULL,
                engine2_name TEXT NOT NULL,
                env_vars TEXT NOT NULL,
                fastchess_params TEXT NOT NULL,
                status TEXT NOT NULL,
                created_at TEXT NOT NULL,
                started_at TEXT,
                finished_at TEXT,
                games_played INTEGER DEFAULT 0,
                games_total INTEGER DEFAULT 0,
                discord_webhook TEXT,
                engine1_bin TEXT,
                engine2_bin TEXT
            );

            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS test_logs (
                id TEXT PRIMARY KEY,
                test_id TEXT NOT NULL,
                timestamp TEXT NOT NULL,
                level TEXT NOT NULL,
                message TEXT NOT NULL,
                FOREIGN KEY (test_id) REFERENCES tests(id)
            );

            CREATE INDEX IF NOT EXISTS idx_test_logs_test_id ON test_logs(test_id);
            "#
        )?;
        Ok(())
    }

    pub fn create_test(&self, test: &Test) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO tests (id, engine1_ref, engine1_name, engine2_ref, engine2_name, 
                env_vars, fastchess_params, status, created_at, discord_webhook)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                &test.id,
                &test.engine1_ref,
                &test.engine1_name,
                &test.engine2_ref,
                &test.engine2_name,
                test.env_vars.to_string(),
                test.fastchess_params.to_string(),
                test.status.as_str(),
                &test.created_at,
                &test.discord_webhook,
            ],
        )?;
        Ok(())
    }

    pub fn get_test(&self, id: &str) -> SqlResult<Option<Test>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, engine1_ref, engine1_name, engine2_ref, engine2_name, 
                    env_vars, fastchess_params, status, created_at, started_at, 
                    finished_at, games_played, games_total, discord_webhook, 
                    engine1_bin, engine2_bin FROM tests WHERE id = ?"
        )?;
        
        match stmt.query_row([id], |row| Ok(row_to_test(row))) {
            Ok(test) => Ok(Some(test)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    pub fn list_tests(&self) -> SqlResult<Vec<Test>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, engine1_ref, engine1_name, engine2_ref, engine2_name, 
                    env_vars, fastchess_params, status, created_at, started_at, 
                    finished_at, games_played, games_total, discord_webhook, 
                    engine1_bin, engine2_bin FROM tests ORDER BY created_at DESC"
        )?;
        
        let tests = stmt.query_map([], |row| Ok(row_to_test(row)))?
            .collect::<Result<Vec<_>, _>>()?;
        
        Ok(tests)
    }

    pub fn update_test_status(&self, id: &str, status: &TestStatus) -> SqlResult<()> {
        self.conn.execute(
            "UPDATE tests SET status = ? WHERE id = ?",
            params![status.as_str(), id],
        )?;
        Ok(())
    }

    pub fn update_test(&self, test: &Test) -> SqlResult<()> {
        self.conn.execute(
            "UPDATE tests SET engine1_ref = ?, engine1_name = ?, engine2_ref = ?, engine2_name = ?,
                env_vars = ?, fastchess_params = ?, status = ?, started_at = ?, finished_at = ?,
                games_played = ?, games_total = ?, discord_webhook = ?, engine1_bin = ?, engine2_bin = ?
             WHERE id = ?",
            params![
                &test.engine1_ref,
                &test.engine1_name,
                &test.engine2_ref,
                &test.engine2_name,
                test.env_vars.to_string(),
                test.fastchess_params.to_string(),
                test.status.as_str(),
                &test.started_at,
                &test.finished_at,
                test.games_played,
                test.games_total,
                &test.discord_webhook,
                &test.engine1_bin,
                &test.engine2_bin,
                &test.id,
            ],
        )?;
        Ok(())
    }

    pub fn delete_test(&self, id: &str) -> SqlResult<()> {
        self.conn.execute("DELETE FROM test_logs WHERE test_id = ?", [id])?;
        self.conn.execute("DELETE FROM tests WHERE id = ?", [id])?;
        Ok(())
    }

    pub fn get_settings(&self) -> SqlResult<Settings> {
        let default_env_vars = self.get_setting("default_env_vars")
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| serde_json::json!({}));

        let default_fastchess_params = self.get_setting("default_fastchess_params")
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_else(|| serde_json::json!({}));

        let compiled_engines_path = self.get_setting("compiled_engines_path")
            .unwrap_or_else(|_| "compiled_engines".to_string());

        let lora_repo_path = self.get_setting("lora_repo_path")
            .unwrap_or_else(|_| ".".to_string());

        let fastchess_path = self.get_setting("fastchess_path")
            .unwrap_or_else(|_| "fastchess".to_string());

        let default_discord_webhook = self.get_setting("default_discord_webhook").ok();

        Ok(Settings {
            default_env_vars,
            default_fastchess_params,
            compiled_engines_path,
            lora_repo_path,
            fastchess_path,
            default_discord_webhook,
        })
    }

    pub fn set_settings(&self, settings: &Settings) -> SqlResult<()> {
        self.set_setting("default_env_vars", settings.default_env_vars.to_string())?;
        self.set_setting("default_fastchess_params", settings.default_fastchess_params.to_string())?;
        self.set_setting("compiled_engines_path", settings.compiled_engines_path.clone())?;
        self.set_setting("lora_repo_path", settings.lora_repo_path.clone())?;
        self.set_setting("fastchess_path", settings.fastchess_path.clone())?;
        if let Some(webhook) = &settings.default_discord_webhook {
            self.set_setting("default_discord_webhook", webhook.clone())?;
        }
        Ok(())
    }

    fn get_setting(&self, key: &str) -> SqlResult<String> {
        self.conn.query_row(
            "SELECT value FROM settings WHERE key = ?",
            [key],
            |row| row.get(0),
        )
    }

    fn set_setting(&self, key: &str, value: String) -> SqlResult<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)",
            params![key, value],
        )?;
        Ok(())
    }

    pub fn add_log(&self, log: &TestLog) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO test_logs (id, test_id, timestamp, level, message) VALUES (?, ?, ?, ?, ?)",
            params![&log.id, &log.test_id, &log.timestamp, &log.level, &log.message],
        )?;
        Ok(())
    }

    pub fn get_logs(&self, test_id: &str, limit: usize) -> SqlResult<Vec<TestLog>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, test_id, timestamp, level, message FROM test_logs 
             WHERE test_id = ? ORDER BY timestamp DESC LIMIT ?"
        )?;
        
        let logs = stmt.query_map(params![test_id, limit as i32], |row| {
            Ok(TestLog {
                id: row.get(0)?,
                test_id: row.get(1)?,
                timestamp: row.get(2)?,
                level: row.get(3)?,
                message: row.get(4)?,
            })
        })?
            .collect::<Result<Vec<_>, _>>()?;
        
        // Reverse to get chronological order
        Ok(logs.into_iter().rev().collect())
    }
}

fn row_to_test(row: &Row) -> Test {
    Test {
        id: row.get(0).unwrap_or_default(),
        engine1_ref: row.get(1).unwrap_or_default(),
        engine1_name: row.get(2).unwrap_or_default(),
        engine2_ref: row.get(3).unwrap_or_default(),
        engine2_name: row.get(4).unwrap_or_default(),
        env_vars: serde_json::from_str(&row.get::<_, String>(5).unwrap_or_default())
            .unwrap_or_else(|_| serde_json::json!({})),
        fastchess_params: serde_json::from_str(&row.get::<_, String>(6).unwrap_or_default())
            .unwrap_or_else(|_| serde_json::json!({})),
        status: TestStatus::from_str(&row.get::<_, String>(7).unwrap_or_default())
            .unwrap_or(TestStatus::Pending),
        created_at: row.get(8).unwrap_or_default(),
        started_at: row.get(9).ok(),
        finished_at: row.get(10).ok(),
        games_played: row.get(11).unwrap_or_default(),
        games_total: row.get(12).unwrap_or_default(),
        discord_webhook: row.get(13).ok(),
        engine1_bin: row.get(14).ok(),
        engine2_bin: row.get(15).ok(),
    }
}
