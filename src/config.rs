use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub users: Vec<UserConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerConfig {
    pub domain: String,
    pub listen_addr: String,
    pub log_dir: String,
}

#[derive(Debug, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;

    fn load_config_from_str(contents: &str) -> Result<Config> {
        let config: Config = toml::from_str(contents)?;
        config.validate()?;
        Ok(config)
    }

    #[test]
    fn load_valid_config() -> Result<()> {
        let contents = r#"
[server]
domain = "example.com"
listen_addr = "127.0.0.1:8080"
log_dir = "/tmp/thor"

[[users]]
name = "alice"
nwcs = ["nwc://example"]
"#;
        let config = load_config_from_str(contents)?;
        assert_eq!(config.server.domain, "example.com");
        assert_eq!(config.server.listen_addr, "127.0.0.1:8080");
        assert_eq!(config.server.log_dir, "/tmp/thor");
        assert_eq!(config.users.len(), 1);
        assert_eq!(config.users[0].name, "alice");
        assert_eq!(config.users[0].nwcs, vec!["nwc://example".to_string()]);
        Ok(())
    }

    #[test]
    fn load_config_rejects_empty_nwcs() {
        let contents = r#"
[server]
domain = "example.com"
listen_addr = "127.0.0.1:8080"
log_dir = "/tmp/thor"

[[users]]
name = "alice"
nwcs = []
"#;
        let res = load_config_from_str(contents);
        assert!(res.is_err());

        let err = res.unwrap_err();
        assert!(
            err.to_string().contains("user alice has no NWC configured"),
            "unexpected error: {err}"
        );
    }
}
