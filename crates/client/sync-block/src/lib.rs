use std::path::Path;
use std::time::Duration;
use std::{env, thread, time};

use base64::engine::general_purpose;
use base64::Engine as _;
use dotenv::dotenv;
use hyper::header::{HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use hyper::{Body, Client, Request};
use lazy_static::lazy_static;
use rocksdb::{Error, IteratorMode, DB};
use serde_json::{json, Value};
use tokio::runtime::Runtime;
use {reqwest, tokio};

// Import Lazy from the lazy_static crate
// Import the Error type from rocksdb crate
// Define a struct to hold the DB instance.
pub struct MyDatabase {
    db: DB,
}

impl MyDatabase {
    // Constructor to open the database.
    fn open() -> Result<MyDatabase, Error> {
        let path = Path::new("epool");
        let db = DB::open_default(&path)?;
        Ok(MyDatabase { db })
    }

    // Method to perform a read operation.
    fn read(&self, key: String) -> String {
        // Serialize key to bytes
        let key_bytes = key.as_bytes();

        // Use the 'get' method to retrieve the value.
        let result_of_get = self.db.get(key_bytes);

        let option = match result_of_get {
            Ok(val) => val,
            Err(err) => {
                eprintln!("Failed to read from DB: {:?}", err);
                None
            }
        };

        // Handle the None case.
        let value_vec = match option {
            Some(val) => val,
            None => Vec::new(),
        };
        let value = String::from_utf8(value_vec).unwrap();

        value
    }

    // Method to perform a write operation.
    pub fn write(&self, key: String, value: String) {
        // Serialize key to bytes
        let key_bytes = key.as_bytes();

        // Serialize value for storing in DB
        let value_bytes = value.as_bytes();
        let result_of_put = self.db.put(key_bytes, value_bytes);
        match result_of_put {
            Ok(()) => {}
            Err(err) => eprintln!("Failed to write to DB: {:?}", err),
        };
    }

    fn clear(&self) {
        // Create an iterator starting at the first key.
        let iter = self.db.iterator(IteratorMode::Start);

        // Iterate through all key-value pairs and print them.
        for result in iter {
            let deleted = self.db.delete(result.unwrap().0);
            match deleted {
                Ok(()) => {}
                Err(err) => {
                    eprintln!("Failed to delete, error: {:?}", err);
                }
            }
        }
    }

    fn display_all(&self) {
        // Create an iterator starting at the first key.
        let iter = self.db.iterator(IteratorMode::Start);

        // Iterate through all key-value pairs and print them.
        for result in iter {
            match result {
                Ok((key, value)) => {
                    // println!("display_all_data: key: {:?} value {:?}", key, value);
                    println!("display_all_data: key: {:?} value.len(): {:?}", key, value.len());
                }
                Err(err) => {
                    eprintln!("There is an error! {:?}", err);
                }
            }
        }
    }

    fn get_next_entry(&self, start_key: String) -> (String, String) {
        // Serialize key to bytes. The process is 2-step since u64 does not directly support as_ref()
        let key_bytes = start_key.as_bytes();

        // Create an iterator starting from the key after the specified start_key.
        let mut iter = self.db.iterator(IteratorMode::From(key_bytes, rocksdb::Direction::Forward));

        // Iterate to get the next entry.
        let key_vec = iter.next().unwrap().unwrap().0.into_vec();
        let value_vec = iter.next().unwrap().unwrap().1.into_vec();
        let key = String::from_utf8(key_vec).unwrap();
        let value = String::from_utf8(value_vec).unwrap();
        return (key, value);
    }
}

// Create a global instance of MyDatabase that can be accessed from other modules.
lazy_static! {
    pub static ref SYNC_DB: MyDatabase = {
        let db = MyDatabase::open().unwrap_or_else(|err| {
            eprintln!("Failed to open database: {:?}", err);
            std::process::exit(1); // Exit the program on error
        });

        db.clear();

        // Perform write operations here
        db.write("sync".to_string(), "0".to_string());
        db.write("sync_target".to_string(), "0".to_string());

        db // Return the initialized MyDatabase
    };
}

fn encode_data_to_base64(original: String) -> String {
    // Convert string to bytes
    let bytes = original.as_bytes();
    // Convert bytes to base64
    let base64_str: String = general_purpose::STANDARD.encode(&bytes);
    base64_str
}

async fn submit_to_da(data: String) -> Result<String, Box<dyn std::error::Error>> {
    dotenv().ok();
    let da_host = env::var("DA_HOST")?;
    let da_namespace = env::var("DA_NAMESPACE")?;
    let da_auth_token = env::var("DA_AUTH_TOKEN")?;
    let da_auth = format!("Bearer {}", da_auth_token);

    let encoded_data = encode_data_to_base64(data);

    let client = reqwest::Client::new();
    let rpc_request = json!({
        "jsonrpc": "2.0",
        "method": "blob.Submit",
        "params": [
            [
                {
                    "namespace": da_namespace,
                    "data": encoded_data,
                }
            ]
        ],
        "id": 1,
    });

    let uri = std::env::var("DA_URI").unwrap_or_else(|_| da_host);

    let resp = client
        .post(&uri)
        .header(reqwest::header::AUTHORIZATION, da_auth)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(rpc_request.to_string())
        .timeout(Duration::from_secs(100))
        .send()
        .await?;

    let response_body = resp.bytes().await?;
    let parsed: Value = serde_json::from_slice(&response_body)?;
    if let Some(result_value) = parsed.get("result") {
        Ok(result_value.to_string())
    } else {
        Err("Result not found in response".into()) // Or create a custom error type
    }
}

pub fn sync_with_da() {
    println!(
        "
        @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
        @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@&&####&&@@@@@@@#&@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
        @@@@@@@@@@@@@@@@@&P?777J5B&@@@@@#G5?!~^^::::::^^~!?5BJ!@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
        @@@@@@@@@@@@@@@@B: :7?7~^:^!J5?~::~7J5PGB####BGP5J7~  :75#@@@@@@@@@@@@@@@@@@@@@@@@@@@@
        @@@@@@@@@@@@@@@@7  G@@@@@B7  .  7B@@@@@@@@@@@@@@@@@Y.55!:.~Y#@@@@@@@@@@@@@@@@@@@@@@@@@
        @@@@@@@@@@@@@@@@J  J@@@B7..7G&#57~!Y#@@@@@@@@@@@@@Y Y@@@&G7..7B@@@@@@@@@@@@@@@@@@@@@@@
        @@@@@@@@@@@@@@@@#:  5#7 :Y#@@@@@@&G?!7P&@@@@@@@@@Y ?@@@@@@@&Y: 7B@@@@@@@@@@@@@@@@@@@@@
        @@@@@@@@@@@@@@@@@G. ...J&@@@@@@@@@@@&GJ!JB@@#PJJ7 ~@@@@@@@@@@&Y..Y@@@@@@@@@@@@@@@@@@@@
        @@@@@@@@@@@@@@@@@@5   !&@@@@@@@@@@@@@@@&P??7.     .?&@@@@@@@@@@#~ !&@@@@@@@@@@@@@@@@@@
        @@@@@@@@@@@@@@@@@&! .  ^G@@@@@@@@@@@@@&&@@J         Y@@@@@@@@@@@@7 ~&@@@@@@@@@@@@@@@@@
        @@@@@@@@@@@@@@@@@7 !G:   ?P5YYJJJJJYYYJJJY?         !YJYYY55PGBB#&7 !@@@@@@@@@@@@@@@@@
        @@@@@@@@@@@@@&BP?  ^!~~    !GBB###&&&&&@@@@G:     .P&##BBGP5YJ?7!!~  ?PB&@@@@@@@@@@@@@
        @@@@@@@@@&P?^::^  ?##&@5.   ~G@@@@@@@@@@@@@5. ?GBBJ?G@@@@@@@@@@@&&#J  ~^:~?P&@@@@@@@@@
        @@@@@@@&J: ~JG&B..B@@@@@#!    !G@@@@@@@@@@Y  !@@@@@#J7G@@@@@@@@@@@@#. B&BY! :J&@@@@@@@
        @@@@@@@7 .G@@@@P .#@@@@@@@5:    7B@@@@@@@Y  ^&@@@@@@@B?7G@@@@@@@@@@&: P@@@@B: 7@@@@@@@
        @@@@@@@J  ?G&@@G .#@@@@@@@@&?.   .7B@@@@Y  :#@@@@@@@@@@G7?#@@@@@@@@&: G@@&B?  J@@@@@@@
        @@@@@@@@P~  :!JY. G@@@@@@@@@@#7.   .7B@Y  .G@@@@@@@@@@@@@P!J&@@@@@@G  YY!:  ~P@@@@@@@@
        @@@@@@@@@@BY!:    .^~7?JY5PGGBB5~    .~   Y@&&&&&&&####BBBP^^YYJ7!~:    :!YB@@@@@@@@@@
        @@@@@@@@@@@@@&BPJ. ..          ..         :::::::::....          .. .JPB&@@@@@@@@@@@@@
        @@@@@@@@@@@@@@@@@Y :GBP5J?!~^:...                    ..:^^~7?J7 ?G: J@@@@@@@@@@@@@@@@@
        @@@@@@@@@@@@@@@@@@J :B@@@@@@@&&##BB7   .     :?PGGGBB##&&@@@@@@Y.: J@@@@@@@@@@@@@@@@@@
        @@@@@@@@@@@@@@@@@@@5. Y@@@@@@@@@@@5   :BG7:   .!5&@@@@@@@@@@@@@5  :#@@@@@@@@@@@@@@@@@@
        @@@@@@@@@@@@@@@@@@@@B~ ~G@@@@@@@@5   .G@@@#5!.   :7G&@@@@@@@@G~ !Y ^#@@@@@@@@@@@@@@@@@
        @@@@@@@@@@@@@@@@@@@@@@P^ ~5&@@@@Y    5@@@@@@@#Y~.   ^JG&@@&P~ ^5@@J ~&@@@@@@@@@@@@@@@@
        @@@@@@@@@@@@@@@@@@@@@@@&P!.:7P&5    J@@@@@@@@@@@#5!:   ^??:.!G@@@@&: 5@@@@@@@@@@@@@@@@
        @@@@@@@@@@@@@@@@@@@@@@@@@@#Y~::    !@@@@@@@@@@@@@@&B?.     :JPB#&&G. Y@@@@@@@@@@@@@@@@
        @@@@@@@@@@@@@@@@@@@@@@@@@@@@@J     :~!7JJYYYYJJ7!^::^!JGGY7^.  .:: .?&@@@@@@@@@@@@@@@@
        @@@@@@@@@@@@@@@@@@@@@@@@@@@@P    :BG5Y?77!~~!77?Y5G#&@@@@@@&#BGP55G#@@@@@@@@@@@@@@@@@@
        @@@@@@@@@@@@@@@@@@@@@@@@@@@@G^..^B@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
        @@@@@@@@@@@@@@@@@@@@@@@@@@@@@&##&@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
        @@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@@
    "
    );

    // Create the runtime
    let rt = match Runtime::new() {
        Ok(rt) => {
            println!("Successfully created runtime");
            rt
        }
        Err(err) => {
            eprintln!("Error in creating runtime: {}", err);
            return;
        }
    };

    loop {
        thread::sleep(time::Duration::from_millis(3000));
        let sync = SYNC_DB.read("sync".to_string());
        let sync_target = SYNC_DB.read("sync_target".to_string());

        println!("sync_target: {:?} and sync {:?}", sync_target, sync);
        if sync_target != sync {
            SYNC_DB.display_all();
            let (next_sync, next_txs) = SYNC_DB.get_next_entry(sync);
            rt.block_on(async {
                let block_height = submit_to_da(next_txs).await;
                match block_height {
                    Ok(block_height) => {
                        println!(
                            "<------------------------------------DA BLOCK \
                             HEIGHT------------------------------------------>: {}",
                            block_height
                        );
                        SYNC_DB.write("sync".to_string(), next_sync);
                    }
                    Err(err) => eprintln!("Failed to submit to DA with error: {:?}", err),
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = 4; //= sumit_to(2, 2);
        assert_eq!(result, 4);
    }
}
