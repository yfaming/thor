use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub users: Vec<UserConfig>,
}

#[derive(Serialize, Deserialize)]
pub struct ServerConfig {
    pub domain: String,
    pub listen_addr: String,
    pub log_dir: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserConfig {
    pub name: String,
    pub nwcs: Vec<String>,
}

impl Config {
    pub fn load_from_toml(config_path: &std::path::Path) -> Result<Config> {
        let config_str = std::fs::read_to_string(config_path)?;
        let config: Config = toml::from_str(&config_str)?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<()> {
        for user_config in &self.users {
            if user_config.nwcs.is_empty() {
                anyhow::bail!("user {} has no NWC configured", user_config.name)
            }
        }
        Ok(())
    }
}
