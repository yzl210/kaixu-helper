use std::collections::HashMap;
use std::time::Duration;

use chrono::Local;
use lazy_static::lazy_static;
use rsteam::steam_user::Summary;
use rsteam::SteamID;
use tokio::sync::Mutex;
use tokio::time::sleep;

use crate::config::CONFIG;
use crate::reply::send_message;

lazy_static! {
    pub(crate) static ref STATUSES: Mutex<HashMap<u64, Summary>> = Mutex::new(HashMap::new());
}


pub(crate) async fn start() {
    let sleep_duration = Duration::from_secs(CONFIG.read().await.refresh_interval);
    loop {
        check().await;
        sleep(sleep_duration).await;
    }
}

async fn check() {
    let mut previous_statuses = STATUSES.lock().await;
    let current_statuses = get_summaries().await;


    for (id, current_summary) in current_statuses.iter() {
        if let Some(previous_summary) = previous_statuses.get(id) {
            let previous_game_info = previous_summary.game_info.clone().unwrap_or_else(|| "".to_string());
            let current_game_info = current_summary.game_info.clone().unwrap_or_else(|| "".to_string());

            if previous_game_info != current_game_info {
                let time = Local::now().format("%Y-%m-%d %H:%M").to_string();
                if current_game_info.is_empty() {
                    send_message(current_summary.profile_name.as_str(), &format!("stopped playing {} on {}", previous_game_info, time), 0xd92121).await;
                } else {
                    send_message(current_summary.profile_name.as_str(), &format!("started playing {} on {}", current_game_info, time), 0x32cd32).await;
                }
            }
        }
    }


    *previous_statuses = current_statuses;
}

async fn get_summaries() -> HashMap<u64, Summary> {
    let client = rsteam::SteamClient::with_api_key(CONFIG.read().await.steam_api_keys.first().unwrap());
    let steam_ids: Vec<SteamID> = CONFIG.read().await.steam_ids.iter().map(|&id| SteamID::from(id)).collect();

    let result = client.get_player_summaries(&steam_ids).await;

    match result {
        Ok(player_summaries) => {
            let mut map = HashMap::new();
            for player in player_summaries {
                map.insert((&player.id).into(), player);
            }
            map
        }
        Err(e) => {
            println!("Error getting player summaries: {:?}", e);
            HashMap::new()
        }
    }
}
