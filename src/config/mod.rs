use anyhow::Result;
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub server_address: String,
    pub jwt_secret: String,
    pub upload_dir: String,
    pub max_file_size: usize,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "sqlite:./data/enterprise_platform.db".to_string());
        
        let server_address = env::var("SERVER_ADDRESS")
            .unwrap_or_else(|_| "0.0.0.0:3000".to_string());
        
        let jwt_secret = env::var("JWT_SECRET")
            .unwrap_or_else(|_| "your-256-bit-secret-key-here-change-in-production".to_string());
        
        let upload_dir = env::var("UPLOAD_DIR")
            .unwrap_or_else(|_| "./uploads".to_string());
        
        let max_file_size = env::var("MAX_FILE_SIZE")
            .unwrap_or_else(|_| "10485760".to_string()) // 10MB default
            .parse::<usize>()
            .unwrap_or(10485760);

        // Ensure upload directory exists
        std::fs::create_dir_all(&upload_dir)?;

        Ok(Config {
            database_url,
            server_address,
            jwt_secret,
            upload_dir,
            max_file_size,
        })
    }
}