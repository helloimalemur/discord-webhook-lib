use reqwest::header::CONTENT_TYPE;
use reqwest::{ClientBuilder};
use serde_json::{json, Result, Value};
use std::process;
use anyhow::{anyhow, Error};

pub async fn send_discord(
    webhook_url: &str,
    message: &str,
    username: &str,
) -> std::result::Result<(), Error> {
    if webhook_url.contains("https://discord.com/api/webhooks/") || webhook_url.contains("https://discordapp.com/api/webhooks/") {
        if let Ok(json_message) = jsonify(username, message) {
            push_message(webhook_url, json_message).await?;
        } else {
            return Err(anyhow!("Unable to Send Discord Webhook"))
        }
    } else {
        return Err(anyhow!("Invalid Discord URL"))
    }
    Ok(())
}

async fn push_message(api_url: &str, json_message: Value) -> reqwest::Result<String> {
    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .no_gzip()
        .build();

    let response = match client {
        Ok(r) => {
            r.post(api_url)
                .header(CONTENT_TYPE, "application/json")
                .json(&json_message)
                .send()
                .await
        }
        Err(_e) => process::exit(3),
    };
    return match response {
        Ok(r) => r.text().await,
        Err(e) => Err(e),
    };
}

pub fn jsonify(username: &str, message: &str) -> Result<Value> {
    let data = json!({
    "username": username,
    "content": message,
    });
    Ok(data)
}

#[cfg(test)]
mod tests {
    use std::env;
    use crate::send_discord;
    /// export DISC_WH="https://discordapp.com/api/webhooks/xxxxxxxxxxxxxxxxxxxxx"
    #[test]
    fn test_send_wh() {
        if let Ok(whu) = env::var("DISC_WH") {
            let rt = tokio::runtime::Runtime::new();
            rt.unwrap().block_on(send_discord(
                whu.as_str(),
                "Hello World",
                "Lazarus"
            )).unwrap();
        }
    }
}
