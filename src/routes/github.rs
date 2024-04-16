use axum::{
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use hmac::{Hmac, Mac};
use serde_json::Value;
use sha2::Sha256;

use crate::config::Config;

type HmacSha256 = Hmac<Sha256>;

pub async fn handle_gh(
    State(config): State<Config>,
    headers: HeaderMap,
    body: Json<Value>,
) -> StatusCode {
    let test = body.to_string();

    println!("{test}");

    if !is_authorized(
        &headers,
        body.to_string().as_bytes(),
        &config.github.webhook_secret,
    ) {
        println!("Auth failed");
        return StatusCode::UNAUTHORIZED;
    }

    if !is_human_user(&body) {
        println!("Human check failed");
        return StatusCode::OK;
    }

    match post_to_webhook(config, body).await {
        Ok(_) => {
            println!("posting to webhook success");
            StatusCode::OK
        }
        Err(_) => {
            println!("posting to webhook fail");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

fn is_authorized(headers: &HeaderMap, body: &[u8], secret: &str) -> bool {
    let signature = match extract_signature(headers) {
        Some(s) => s,
        None => return false,
    };

    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };

    mac.update(body);
    let result = mac.finalize();
    let expected_signature = hex::encode(result.into_bytes());

    signature == format!("sha256={}", expected_signature)
}

fn extract_signature(headers: &HeaderMap) -> Option<String> {
    println!("extract_signature fn called");

    headers
        .get("x-hub-signature-256")
        .and_then(|hv| hv.to_str().ok())
        .map(|s| s.trim_start_matches("sha256=").to_string())
}

fn is_human_user(json: &Value) -> bool {
    println!("is_human_user fn called");

    json.get("sender")
        .and_then(|sender| sender.get("type"))
        .and_then(|user_type| user_type.as_str())
        .map_or(false, |user_type| user_type == "User")
}

async fn post_to_webhook(config: Config, json: Json<Value>) -> anyhow::Result<()> {
    println!("post_to_webhook fn called");

    reqwest::Client::new()
        .post(config.github.target_webhook)
        .json(&json.0)
        .send()
        .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash() {
        let secret = "It's a Secret to Everybody";
        let payload = "Hello, World!";

        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(payload.as_bytes());
        let result = hex::encode(mac.finalize().into_bytes());

        let expected = "757107ea0eb2509fc211221cce984b8a37570b6d7586c22c46f4379c8b043e17";

        assert_eq!(result, expected);
    }
}
