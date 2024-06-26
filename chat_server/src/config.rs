use std::{
    fmt::Display,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::Deserialize;
use tokio::{fs::File, io::AsyncReadExt};

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    pub listen: ListenConfig,
    pub db: DbConfig,
    pub auth: AuthConfig,
    pub file: FileConfig,
}

#[derive(Debug, Default, Deserialize)]
pub struct ListenConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Default, Deserialize)]
pub struct DbConfig {
    pub url: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct AuthConfig {
    pub sk: String,
    pub pk: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct FileConfig {
    pub base_dir: PathBuf,
}

impl Config {
    pub async fn try_new(config_path: impl AsRef<Path> + Display) -> Result<Self> {
        let mut file = File::open(&config_path)
            .await
            .with_context(|| format!("try to open config file from {}", &config_path))?;
        let mut dst = String::new();
        file.read_to_string(&mut dst)
            .await
            .context("read str from config file failed")?;
        let config = serde_yaml::from_str(dst.as_str())
            .context("deserialize str to config object failed")?;
        Ok(config)
    }
}
