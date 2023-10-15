use std::collections::HashMap;
use std::{env, fs};

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

lazy_static! {
    pub static ref DA_CONFIG: HashMap<String, String> = {

    let args: Vec<String> = env::args().collect();
    let mut base_path:String=".".to_string() ;
    // The first argument (at index 0) is the name of the program itself.
    // The actual command-line arguments start from index 1.
    if args.len() > 1 {
        println!("Command-line arguments:");
        for (index, arg) in args.iter().enumerate().skip(1) {
            if arg=="--base-path" && index+1<args.len() {
            base_path = Some(args[index + 1].clone()).unwrap();
            println!("Arg {}", base_path);
            }
        }
    } else {
        println!("No command-line arguments provided.");
    }
    let file_path = format!("{}/chains/dev/configs/Config.toml", base_path);

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

        let da_config = config.try_deserialize::<HashMap<String, String>>().unwrap();

        da_config
    };
}
