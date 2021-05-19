use serde::{Deserialize, Serialize};
use jarvis_lib::EntityType;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub location: String,
    pub entity_type: EntityType,
    pub entity_name: String,
}
