// https://docs.rs/serde_json/latest/serde_json/
use serde_json::{json, Result, Value};
use reqwest::ClientBuilder;
use std::process;
use reqwest::header::{CONTENT_TYPE};


pub async fn send_discord(discord: String, message: String, username: String) {
    let message = format!("{}", message);
    if discord.contains("https://discord.com/api/webhooks/") {
        send(discord.as_str(), username.as_str(), message).await;
    } else {
        println!("invalid discord url");
    }

}

pub async fn send(api_url: &str, username: &str, message: String) {

    let json_message = match jsonify(username, message) {
        Ok(j) => j,
        Err(_e) => process::exit(3)
    };
    push_message(api_url, json_message).await;

}


async fn push_message(api_url: &str, json_message: Value) {
    let client = ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .no_gzip()
        .build();

    let response = match client {
        Ok(r) => r
            .post(api_url)
            .header(CONTENT_TYPE, "application/json")
            .json(&json_message)
            .send().await,
        Err(_e) => process::exit(3)
    };
    let result_text = match response {
        Ok(r) => r.text().await,
        Err(_e) => process::exit(3)
    };
    println!("{:?}", result_text)
}



pub fn jsonify(username: &str, message: String) -> Result<Value> {
    let data = json!({
    "username": username,
    "content": message,
    });


    println!("{}", data.to_string());

    Ok(data)
}
