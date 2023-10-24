use std::fs;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::rule::ReplyRule;

const CONFIG_NAME: &str = "config.yml";

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Config {
    pub(crate) discord_token: String,
    pub(crate) discord_guild_id: u64,
    pub(crate) steam_status_discord_channel_id: u64,
    pub(crate) steam_api_keys: Vec<String>,
    pub(crate) steam_ids: Vec<u64>,
    pub(crate) refresh_interval: u64,
    pub(crate) reply_rules: Vec<ReplyRule>,
}



lazy_static! {
    pub(crate) static ref CONFIG: RwLock<Config> = {
        if fs::metadata(CONFIG_NAME).is_ok() {
                    let contents = fs::read_to_string(CONFIG_NAME)
            .expect("Could not read config");
            let config: Config = serde_yaml::from_str(&contents).expect("Error parsing config");
            return RwLock::new(config);
        }

        let config = Config {
            discord_token: "".to_string(),
            discord_guild_id: 0,
            steam_status_discord_channel_id: 0,
            steam_api_keys: vec![],
            steam_ids: vec![],
            refresh_interval: 30,
            reply_rules: vec![],
        };
        let contents = serde_yaml::to_string(&config).expect("Error serializing config");
        fs::write(CONFIG_NAME, contents).expect("Could not write default config");
        RwLock::new(config)
    };
}

pub(crate) async fn save() {
    let contents = serde_yaml::to_string(&*CONFIG.read().await).expect("Error serializing config");
    fs::write(CONFIG_NAME, contents).expect("Could not save config");
}

pub(crate) async fn reload() {
    let contents = fs::read_to_string(CONFIG_NAME)
        .expect("Could not reload the config file");
    let config: Config = serde_yaml::from_str(&contents).expect("Error parsing config");
    let mut config_write = CONFIG.write().await;
    *config_write = config;
}