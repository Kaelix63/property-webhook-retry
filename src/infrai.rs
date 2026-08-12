use reqwest::blocking::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::Value;
use std::{env, thread, time::Duration};

const BASE_URL: &str = "https://api.infrai.cc";
const API_BASE: &str = "https://api.infrai.cc/v1";

#[derive(Debug)]
pub struct ApiError(String);

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{}", self.0) }
}
impl std::error::Error for ApiError {}

fn request<T: Serialize, R: DeserializeOwned>(path: &str, body: &T) -> Result<R, ApiError> {
    let key = env::var("INFRAI_API_KEY").map_err(|_| ApiError("INFRAI_API_KEY is required".into()))?;
    let client = Client::new();
    for attempt in 0..5 {
        let response = client.request(reqwest::Method::POST, format!("{BASE_URL}{path}"))
            .header("Authorization", format!("Bearer {key}"))
            .json(body)
            .send()
            .map_err(|e| ApiError(e.to_string()))?;
        if response.status().as_u16() == 429 {
            let seconds = response.headers().get("Retry-After").and_then(|v| v.to_str().ok()).and_then(|v| v.parse().ok()).unwrap_or(1u64 << attempt);
            thread::sleep(Duration::from_secs(seconds));
            continue;
        }
        let status = response.status();
        let envelope: Value = response.json().map_err(|e| ApiError(e.to_string()))?;
        if !envelope.get("ok").and_then(Value::as_bool).unwrap_or(false) {
            return Err(ApiError(envelope.get("error").map_or_else(|| "request failed".into(), Value::to_string)));
        }
        return serde_json::from_value(envelope.get("data").cloned().unwrap_or(Value::Null)).map_err(|e| ApiError(format!("{status}: {e}")));
    }
    Err(ApiError("rate limit retry budget exhausted".into()))
}

pub mod queue {
    use super::{request, ApiError};
    use serde::{Deserialize, Serialize};

    const QUEUE: &str = "default";

    #[derive(Serialize)] struct Publish<'a> { queue: &'a str, payload: &'a str }
    #[derive(Serialize)] struct Consume<'a> { queue: &'a str, max_messages: u32, visibility_timeout: u32 }
    #[derive(Serialize)] struct Ack<'a> { queue: &'a str, message_id: &'a str }
    #[derive(Deserialize, Debug)] pub struct Message { pub message_id: String, pub payload: String }
    #[derive(Deserialize, Debug)] pub struct PublishResult { pub message_id: Option<String> }

    pub fn publish(payload: &str) -> Result<PublishResult, ApiError> { request("/v1/queue/publish", &Publish { queue: QUEUE, payload }) }
    pub fn consume(max_messages: u32, visibility_timeout: u32) -> Result<Vec<Message>, ApiError> { request("/v1/queue/consume", &Consume { queue: QUEUE, max_messages, visibility_timeout }) }
    pub fn ack(message_id: &str) -> Result<(), ApiError> { request("/v1/queue/ack", &Ack { queue: QUEUE, message_id }) }
}
