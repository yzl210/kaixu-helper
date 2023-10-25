use std::collections::HashMap;
use std::time::Duration;

use chrono::Utc;
use chrono_tz::Tz;
use lazy_static::lazy_static;
use rsteam::steam_user::{Status, Summary};
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

            let timezone: Tz = CONFIG.read().await.timezone.parse().expect("Invalid timezone");
            let time = Utc::now().with_timezone(&timezone).format("%Y-%m-%d %H:%M").to_string();
            if previous_game_info != current_game_info {
                if current_game_info.is_empty() {
                    send_message(&format!("{}  |  {}", current_summary.profile_name, previous_game_info),  &format!("Stopped Playing\n{}", time), 0xd92121).await;
                } else {
                    send_message(&format!("{}  |  {}", current_summary.profile_name, current_game_info),  &format!("Started Playing\n{}", time), 0x32cd32).await;
                }
            }


            let previous_status = status(&previous_summary.status);
            let current_status = status(&current_summary.status);

            if previous_status != current_status {
                let mut status_msg = "";
                let mut color: u32 = 0xffffff;

                if previous_status == status(&Status::Online) && current_status == status(&Status::Offline) {
                    status_msg = "Offline";
                    color = 0x656566;
                }
                if current_status == status(&Status::Online) {
                    status_msg = if previous_status == status(&Status::Offline) { "Online" } else { "Back" };
                    color = 0x6ecff6;
                }

                if current_status == status(&Status::Busy) {
                    status_msg = "Busy";
                    color = 0xff9900;
                }

                if current_status == status(&Status::Away) {
                    status_msg = "Away";
                    color = 0xfff200;
                }

                if current_status == status(&Status::Snooze) {
                    status_msg = "Snooze";
                    color = 0xfa983a;
                }

                let s = current_status.to_string().clone();
                if status_msg.is_empty() {
                    status_msg = s.as_str();
                }

                send_message(&format!("{}  |  {}", current_summary.profile_name, status_msg),  &time, color).await;
            }

            let previous_name = &previous_summary.profile_name;
            let current_name = &current_summary.profile_name;
            if previous_name != current_name {
                send_message(&format!("{}  |  Name Change", previous_name),  &format!("{}  ->  {}\n{}", previous_name, current_name, &time), 0x32cd32).await;
            }

        }
    }


    *previous_statuses = current_statuses;
}

fn status(status: &Status) -> u8 {
    match status {
        Status::Offline => 0,
        Status::Online => 1,
        Status::Busy => 2,
        Status::Away => 3,
        Status::Snooze => 4,
        Status::LookingToTrade => 5,
        Status::LookingToPlay => 6
    }
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
