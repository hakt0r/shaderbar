pub const CONFIG_DIR: &str = "shaderbar";
pub const CONFIG_FILE: &str = "config.json";
pub const CONFIG_STYLESHEET: &str = "theme.css";

crate::utils::global_init_async!(config, Config, init_config);
pub struct Config {
    pub config: serde_json::Value,
}

pub async fn init_config() -> Config {
    let home = dirs::home_dir().unwrap();
    let config_dir = home.join(CONFIG_DIR);
    let config_file: std::path::PathBuf = config_dir.join(CONFIG_FILE);
    let config_file_clone1: std::path::PathBuf = config_dir.join(CONFIG_FILE);
    let config_file_clone2: std::path::PathBuf = config_dir.join(CONFIG_FILE);

    if !config_dir.exists() {
        tokio::fs::create_dir_all(config_dir).await.unwrap();
    }

    if !config_file.exists() {
        let default_config = include_str!("defaults.json");
        tokio::fs::write(config_file, default_config).await.unwrap();
        println!("Config file created at: {:?}", config_file_clone1);
    }

    let config = tokio::fs::read_to_string(config_file_clone2).await.unwrap();
    let config: serde_json::Value = serde_json::from_str(&config).unwrap();
    println!("Config: {:?}", config);

    return Config { config };
}
