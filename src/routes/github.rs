use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    Json,
};
use hmac::{Hmac, Mac};
use serde_json::Value;
use sha2::Sha256;
use subtle::ConstantTimeEq;

use crate::config::Config;

type HmacSha256 = Hmac<Sha256>;

pub async fn handle_gh(
    State(config): State<Config>,
    headers: HeaderMap,
    body: Bytes,
) -> StatusCode {
    tracing::info!("Received POST request at /github.");

    let body_bytes = body.as_ref();

    if !is_authorized(&headers, body_bytes, &config.github.webhook_secret) {
        tracing::warn!("Unauthorized request at /github!");
        return StatusCode::UNAUTHORIZED;
    }

    let json: Value = match serde_json::from_slice(body_bytes) {
        Ok(json) => json,
        Err(_) => {
            tracing::warn!("Wrong formatted request at /github!");
            return StatusCode::BAD_REQUEST;
        }
    };

    if !is_human_user(&json) {
        return StatusCode::OK;
    }

    match post_to_webhook(config, body, headers).await {
        Ok(_) => {
            tracing::info!("Forwarded github event to webhook.");
            StatusCode::OK
        }
        Err(e) => {
            tracing::info!("Failed to forward github event: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

fn is_authorized(headers: &HeaderMap, body: &[u8], secret: &str) -> bool {
    let header_signature = match extract_signature(headers) {
        Some(s) => s,
        None => return false,
    };

    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };

    mac.update(body);
    let calculated_signature = mac.finalize().into_bytes();

    let header_signature = match hex::decode(header_signature) {
        Ok(s) => s,
        Err(_) => return false,
    };

    header_signature.ct_eq(&calculated_signature).into()
}

fn extract_signature(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-hub-signature-256")
        .and_then(|hv| hv.to_str().ok())
        .map(|s| s.trim_start_matches("sha256=").to_string())
}

fn is_human_user(json: &Value) -> bool {
    json.get("sender")
        .and_then(|sender| sender.get("type"))
        .and_then(|user_type| user_type.as_str())
        .map_or(false, |user_type| user_type == "User")
}

async fn post_to_webhook(config: Config, body: Bytes, headers: HeaderMap) -> anyhow::Result<()> {
    let res = reqwest::Client::new()
        .post(config.github.target_webhook)
        .headers(headers)
        .body(body)
        .send()
        .await?;

    println!("{:#?}", res);

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
