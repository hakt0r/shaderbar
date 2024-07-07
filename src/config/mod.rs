use crate::utils::*;
use dirs::home_dir;
use serde_json::{from_str, Value};
use std::path::PathBuf;
use tokio::fs::{create_dir_all, write};

pub const CONFIG_DIR: &str = ".config/shaderbar";
pub const CONFIG_FILE: &str = "config.json";
pub const CONFIG_STYLESHEET: &str = "theme.css";

global_init_async!(config, Config, init_config);
pub struct Config {
    pub config: serde_json::Value,
    pub config_dir: PathBuf,
    pub config_file: PathBuf,
    pub stylesheet_file: PathBuf,
}

pub async fn init_config() -> Config {
    let home = dirs::home_dir().unwrap();
    let config_dir = home.join(CONFIG_DIR);
    let config_file: PathBuf = config_dir.join(CONFIG_FILE);
    let stylesheet_file: PathBuf = config_dir.join(CONFIG_STYLESHEET);

    create_config_dir().await;
    write_default_config().await;
    write_default_styles().await;

    return Config {
        config: read_config().await,
        config_dir,
        config_file,
        stylesheet_file,
    };
}

fn dir() -> PathBuf {
    home_dir().unwrap().join(CONFIG_DIR)
}

fn file() -> PathBuf {
    home_dir().unwrap().join(CONFIG_DIR).join(CONFIG_FILE)
}

fn styles() -> PathBuf {
    home_dir().unwrap().join(CONFIG_DIR).join(CONFIG_STYLESHEET)
}

async fn create_config_dir() {
    early_return!(!dir().exists());
    create_dir_all(dir()).await.unwrap();
}

async fn write_default_config() {
    let config_file = file();
    early_return!(!dir().exists());
    early_return!(config_file.exists());
    let default_config = include_str!("defaults.json");
    write(config_file.clone(), default_config).await.unwrap();
    println!("Config file created at: {:?}", config_file.clone());
}

async fn write_default_styles() {
    let stylesheet_file = styles();
    early_return!(!dir().exists());
    early_return!(stylesheet_file.exists());
    let default_stylesheet = include_str!("defaults.css");
    write(stylesheet_file.clone(), default_stylesheet)
        .await
        .unwrap();
    println!("Stylesheet file created at: {:?}", stylesheet_file.clone());
}

async fn read_config() -> Value {
    let config_file = file();
    let config = tokio::fs::read_to_string(config_file.clone())
        .await
        .unwrap();
    let config: Value = from_str(&config).unwrap();
    return config;
}
