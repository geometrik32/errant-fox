use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tracing::{error, info};

#[derive(Deserialize, Debug)]
struct VkError {
    error_code: i32,
    error_msg: String,
}

#[derive(Deserialize, Debug)]
struct VkResponse {
    response: Option<serde_json::Value>,
    error: Option<VkError>,
}

pub struct VkNotificationService {
    client: Client,
    group_token: Option<String>,
    api_version: String,
    // Map of (user_id, video_id) -> last outcome notification Instant
    outcome_throttles: Mutex<HashMap<(String, String), Instant>>,
}

impl VkNotificationService {
    pub fn new(group_token: Option<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(Duration::from_secs(10))
                .build()
                .unwrap_or_default(),
            group_token,
            api_version: "5.131".to_string(),
            outcome_throttles: Mutex::new(HashMap::new()),
        }
    }

    /// Check if we should send an outcome notification to the user for this video.
    /// Returns `true` if notification is allowed (and updates the last sent time),
    /// or `false` if it is throttled.
    pub fn check_outcome_throttle(&self, user_id: &str, video_id: &str) -> bool {
        let mut throttles = self.outcome_throttles.lock().unwrap();
        let key = (user_id.to_string(), video_id.to_string());
        let now = Instant::now();
        if let Some(&last_time) = throttles.get(&key) {
            if now.duration_since(last_time) < Duration::from_secs(30 * 60) {
                return false;
            }
        }
        throttles.insert(key, now);
        true
    }

    /// Send a notification message to a VK User ID.
    /// Since the network calls are async, this runs in the background.
    pub async fn send_notification(&self, vk_id: &str, message: &str) {
        let token = match &self.group_token {
            Some(t) if !t.trim().is_empty() => t,
            _ => {
                info!("VK notifications are disabled (no token provided)");
                return;
            }
        };

        // Parse numeric user_id
        let user_id: i64 = match vk_id.trim().parse() {
            Ok(id) => id,
            Err(_) => {
                error!("Invalid numeric VK ID: '{}'. Must be numeric.", vk_id);
                return;
            }
        };

        // vk api random_id is used to prevent duplicate message deliveries
        let random_id = (uuid::Uuid::new_v4().as_u128() & 0x7FFFFFFF) as i32;

        let params = [
            ("user_id", user_id.to_string()),
            ("random_id", random_id.to_string()),
            ("message", message.to_string()),
            ("access_token", token.clone()),
            ("v", self.api_version.clone()),
        ];

        match self.client
            .post("https://api.vk.com/method/messages.send")
            .form(&params)
            .send()
            .await
        {
            Ok(resp) => {
                match resp.json::<VkResponse>().await {
                    Ok(vk_resp) => {
                        if let Some(err) = vk_resp.error {
                            error!("VK API returned error: code={}, msg='{}'", err.error_code, err.error_msg);
                        } else {
                            info!("VK notification sent successfully to {}", user_id);
                        }
                    }
                    Err(e) => {
                        error!("Failed to parse VK response JSON: {}", e);
                    }
                }
            }
            Err(e) => {
                error!("Failed to send HTTP request to VK API: {}", e);
            }
        }
    }
}
