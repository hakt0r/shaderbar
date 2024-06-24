pub const CONFIG_DIR: &str = ".config/shaderbar";
pub const CONFIG_FILE: &str = "config.json";
pub const CONFIG_STYLESHEET: &str = "theme.css";

crate::utils::global_init_async!(config, Config, init_config);
pub struct Config {
    pub config: serde_json::Value,
    pub config_dir: std::path::PathBuf,
    pub config_file: std::path::PathBuf,
    pub stylesheet_file: std::path::PathBuf,
}

pub async fn init_config() -> Config {
    let home = dirs::home_dir().unwrap();
    let config_dir = home.join(CONFIG_DIR);
    let config_file: std::path::PathBuf = config_dir.join(CONFIG_FILE);
    let stylesheet_file: std::path::PathBuf = config_dir.join(CONFIG_STYLESHEET);

    if !config_dir.clone().exists() {
        tokio::fs::create_dir_all(config_dir.clone()).await.unwrap();
    }

    if !config_file.clone().exists() {
        let default_config = include_str!("defaults.json");
        tokio::fs::write(config_file.clone(), default_config)
            .await
            .unwrap();
        println!("Config file created at: {:?}", config_file.clone());
    }

    let config = tokio::fs::read_to_string(config_file.clone())
        .await
        .unwrap();
    let config: serde_json::Value = serde_json::from_str(&config).unwrap();

    if !stylesheet_file.clone().exists() {
        let default_stylesheet = include_str!("defaults.css");
        tokio::fs::write(stylesheet_file.clone(), default_stylesheet)
            .await
            .unwrap();
        println!("Stylesheet file created at: {:?}", stylesheet_file);
    }

    return Config {
        config,
        config_dir,
        config_file,
        stylesheet_file,
    };
}
