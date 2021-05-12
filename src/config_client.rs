use crate::model::Config;
use serde_yaml;
use std::env;
use std::error::Error;
use std::fs;

pub struct ConfigClientConfig {
    config_path: String,
}

impl ConfigClientConfig {
    pub fn new(config_path: String) -> Result<Self, Box<dyn Error>> {
        println!("ConfigClientConfig::new(config_path: {})", config_path);
        Ok(Self { config_path })
    }

    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        let config_path = env::var("CONFIG_PATH").unwrap_or("/configs/config.yaml".to_string());

        Self::new(config_path)
    }
}

pub struct ConfigClient {
    config: ConfigClientConfig,
}

impl ConfigClient {
    pub fn new(config: ConfigClientConfig) -> Self {
        Self { config }
    }

    pub fn read_config_from_file(&self) -> Result<Config, Box<dyn Error>> {
        println!("Loading config from {}...", &self.config.config_path);

        let config_file_contents = fs::read_to_string(&self.config.config_path)?;
        let config: Config = serde_yaml::from_str(&config_file_contents)?;

        println!(
            "Loaded config from {}: {:?}",
            &self.config.config_path, &config
        );

        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::EntityType;

    #[test]
    fn read_config_from_file_returns_deserialized_test_file() {
        let config_client =
            ConfigClient::new(ConfigClientConfig::new("test-config.yaml".to_string()).unwrap());

        let config = config_client.read_config_from_file().unwrap();

        assert_eq!(config.location, "My Home".to_string());
        assert_eq!(config.entity_type, EntityType::Device);
        assert_eq!(config.entity_name, "TP-Link HS110".to_string());
    }
}
