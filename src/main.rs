extern crate chrono;

use reqwest::Client;
use systemstat::{System, Platform};
use serde::Deserialize;
use serde_json::json;
use futures::{stream, StreamExt};
use std::time::{Duration};
use tokio::time;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), ()> {
    let mut config: Config = serde_json::from_str(&fs::read_to_string("config.json")
        .expect("Failed to read config file.")).expect("Failed to config.");
    {
        let sys = System::new();
        let client = Client::new();
        client.post(config.url.clone() + "/updatePersistent")
            .body(serde_json::to_vec(&json!({
            "physicalRam": sys.memory().expect("h").total.as_u64()
        })).expect("Failed to serialize."))
            .send().await.expect("Request failed.");
        client.post(config.url.clone() + "/updateOther")
            .body(serde_json::to_vec(&json!({
            "bootTimestamp": sys.boot_time().expect("h").naive_utc().timestamp()
        })).expect("Failed to serialize."))
            .send().await.expect("Request failed.");
    }
    if config.update_interval == 0 { config.update_interval = 2; }
    let interval = time::interval(Duration::from_secs(config.update_interval));

    let forever = stream::unfold(interval, |mut interval| async {
        interval.tick().await;
        let sys = System::new();
        let client = Client::new();
        let mem = sys.memory().expect("h");
        client.post(config.url.clone() + "/updateMemory")
            .body(serde_json::to_vec(&json!({
                "used": mem.total.as_u64() - mem.free.as_u64(),
                "free": mem.free.as_u64()
        })).expect("Failed to serialize."))
            .send().await.expect("failed to req");
        Some(((), interval))
    });

    forever.for_each(|_| async {}).await;
    Ok(())
}

#[derive(Deserialize, Debug)]
struct Config {
    url: String,
    update_interval: u64,
}