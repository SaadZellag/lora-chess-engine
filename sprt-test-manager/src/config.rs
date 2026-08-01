use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Config {
    pub db_path: String,
    pub host: String,
    pub port: u16,
}

impl Config {
    pub fn load() -> Result<Self> {
        Ok(Config {
            db_path: std::env::var("DB_PATH").unwrap_or_else(|_| "sprt_tests.db".to_string()),
            host: std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or(8000),
        })
    }
}
