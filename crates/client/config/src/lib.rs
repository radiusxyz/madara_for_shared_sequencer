use std::collections::HashMap;
use std::fs;

use config::{Config, Environment, File};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use toml;

#[derive(Serialize, Deserialize)]
pub struct DaConfig {
    sequencer_private_key: String,
    is_validating: bool,
    host: String,
    namespace: String,
    auth_token: String,
}

impl Default for DaConfig {
    fn default() -> Self {
        DaConfig {
            sequencer_private_key: "0x00c1cf1490de1352865301bb8705143f3ef938f97fdf892f1090dcb5ac7bcd1d".to_string(),
            is_validating: false,
            host: "http://localhost:26658".to_string(),
            namespace: "AAAAAAAAAAAAAAAAAAAAAAAAAAECAwQFBgcICRA=".to_string(),
            auth_token: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.\
                         eyJBbGxvdyI6WyJwdWJsaWMiLCJyZWFkIiwid3JpdGUiLCJhZG1pbiJdfQ.\
                         R25jC3ptCU5PQfvpRUJSME6k0RSP6h97NDq44oAndSs"
                .to_string(),
        }
    }
}

fn save_config<T: Serialize>(config: &T, filename: &str) {
    let toml = toml::to_string_pretty(config).expect("Failed to serialize to TOML");
    fs::write(filename, toml).expect("Failed to write to Config.toml");
}

fn file_exists(filename: &str) -> bool {
    fs::metadata(filename).is_ok()
}

lazy_static! {
    pub static ref DA_CONFIG: HashMap<String, String> = {
        if !file_exists("Config.toml") {
            // Config file doesn't exist, create it with default values
            let config = DaConfig::default();
            save_config(&config, "Config.toml");
        }
        let config = Config::builder()
            .add_source(File::with_name("Config.toml"))
            .add_source(Environment::with_prefix("da"))
            .build()
            .unwrap();

        let da_config = config.try_deserialize::<HashMap<String, String>>().unwrap();

        da_config
    };
}
