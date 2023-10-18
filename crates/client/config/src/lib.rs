use std::collections::HashMap;
use std::fs;
use std::mem::MaybeUninit;
use std::sync::{Mutex, Once};

use config::{Config, Environment, File};
use serde::{Deserialize, Serialize};
use toml;

pub static mut CONFIG: MaybeUninit<ConfigMap> = MaybeUninit::uninit();
static INIT: Once = Once::new();

pub fn config_map() -> &'static ConfigMap {
    if INIT.is_completed() {
        unsafe { CONFIG.assume_init_ref() }
    } else {
        panic!("ConfigMap is not initialized..");
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigMap {
    inner: Mutex<HashMap<String, String>>,
}

impl Default for ConfigMap {
    fn default() -> Self {
        Self { inner: Mutex::new(HashMap::default()) }
    }
}

impl ConfigMap {
    pub fn init(path: &String) {
        let file_path = format!("{}/Config.toml", path);

        if let Some(parent_dir) = file_path.rfind('/') {
            if let Err(e) = fs::create_dir_all(&file_path[0..parent_dir]) {
                eprintln!("Failed to create directory: {:?}", e);
            }
        }

        if !file_exists(&file_path) {
            // Config file doesn't exist, create it with default values
            let config = DaConfig::default();
            save_config(&config, &file_path);
        }

        let config = Config::builder()
            .add_source(File::with_name(&file_path))
            .add_source(Environment::with_prefix("da"))
            .build()
            .unwrap();

        let config: HashMap<String, String> = config.try_deserialize::<HashMap<String, String>>().unwrap();

        unsafe {
            INIT.call_once(|| {
                let config_map = ConfigMap::default();
                config.iter().for_each(|(key, value)| {
                    config_map.insert(key, value);
                });
                CONFIG.write(config_map);
            });
        }
    }

    pub fn get(&self, key: impl AsRef<str>) -> Option<String> {
        let map_guard = self.inner.lock().unwrap();
        map_guard.get(key.as_ref()).cloned()
    }

    pub fn insert(&self, key: impl ToString, value: impl ToString) {
        let mut map_guard = self.inner.lock().unwrap();
        map_guard.insert(key.to_string(), value.to_string());
    }
}

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
            host: "".to_string(),
            namespace: "".to_string(),
            auth_token: "".to_string(),
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
