use reqwest::header::CONTENT_TYPE;
use reqwest::ClientBuilder;
use serde_json::{json, Result, Value};
use std::process;

pub async fn send_discord(
    webhook_url: &str,
    message: &str,
    username: &str,
) -> std::result::Result<(), reqwest::Error> {
    if webhook_url.contains("https://discord.com/api/webhooks/") {
        let json_message = match jsonify(username, message) {
            Ok(j) => j,
            Err(_e) => process::exit(3),
        };
        push_message(webhook_url, json_message).await?;
    } else {
        println!("invalid discord url");
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

// #[cfg(test)]
// mod tests {
//     use crate::send_discord;
//
//     #[test]
//     fn test_send_wh() {
//         let rt = tokio::runtime::Runtime::new();
//         rt.unwrap().block_on(send_discord(
//             "webhook_url",
//             "Hello World",
//             "Lazarus"
//         )).unwrap();
//     }
// }
