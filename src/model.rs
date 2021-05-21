use jarvis_lib::config_client::SetDefaults;
use jarvis_lib::model::EntityType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub location: String,
    pub entity_type: EntityType,
    pub entity_name: String,
}

impl SetDefaults for Config {
    fn set_defaults(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use jarvis_lib::config_client::{ConfigClient, ConfigClientConfig};
    use jarvis_lib::model::EntityType;

    #[test]
    fn read_config_from_file_returns_deserialized_test_file() {
        let config_client =
            ConfigClient::new(ConfigClientConfig::new("test-config.yaml".to_string()).unwrap());

        let config: Config = config_client.read_config_from_file().unwrap();

        assert_eq!(config.location, "My Home".to_string());
        assert_eq!(config.entity_type, EntityType::Device);
        assert_eq!(config.entity_name, "TP-Link HS110".to_string());
    }
}
