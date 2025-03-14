use mime;
use reqwest::{multipart::{Form, Part}, Client};
use std::fs;
use std::path::Path;

pub struct DiscordMessage {
    client: Client,
    webhook_url: String,
    gif_path: Option<String>,
    message: Option<String>,
}

impl DiscordMessage {
    pub fn builder(webhook_url: impl Into<String>) -> DiscordMessageBuilder {
        DiscordMessageBuilder {
            webhook_url: webhook_url.into(),
            gif_path: None,
            message: None,
        }
    }

    pub async fn send(&self) -> Result<(), Box<dyn std::error::Error>> {

        match self.gif_path {
            // None => {
            //     let form = Form::new().part("file", part);
            //
            //     let res = self.client
            //         .post(&self.webhook_url)
            //         .multipart(form)
            //         .send()
            //         .await?;
            //
            //     if res.status().is_success() {
            //         println!("Webhook sent successfully!");
            //     } else {
            //         println!("Failed to send webhook. HTTP Status: {}", res.status());
            //     }
            //
            // }
            Some(_) => {
                let gif_path = self.gif_path.as_ref().ok_or("No GIF file path specified")?;

                let file_bytes = fs::read(gif_path)?;
                let filename = Path::new(gif_path)
                    .file_name()
                    .map(|os_str| os_str.to_string_lossy().to_string())
                    .ok_or("Invalid file name")?;

                let part = Part::bytes(file_bytes)
                    .file_name(filename.clone())
                    .mime_str("image/gif")?;

                let body = format!("{{\"username\": \"zm\", \"content\": event => {}\"\"}}", filename);

                let form = Form::new().part("file1", part);

                let res = self.client
                    .post(&self.webhook_url)
                    .body(body)
                    .multipart(form)
                    .send()
                    .await?;

                if res.status().is_success() {
                    println!("Webhook sent successfully!");
                } else {
                    println!("Failed to send webhook. HTTP Status: {}", res.status());
                }

            }
            _ => {}
        }

        Ok(())
    }
}

pub struct DiscordMessageBuilder {
    webhook_url: String,
    gif_path: Option<String>,
    message: Option<String>
}

impl DiscordMessageBuilder {
    pub fn gif_path(&mut self, path: impl Into<String>) {
        self.gif_path = Some(path.into());
    }

    pub fn add_message(&mut self, message: impl Into<String>) {
        self.message = Option::from(message.into());
    }

    pub fn build(self) -> DiscordMessage {
        DiscordMessage {
            client: Client::new(),
            webhook_url: self.webhook_url,
            gif_path: self.gif_path,
            message: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::message::DiscordMessage;
    use std::env;

    #[test]
    fn send_test() {
        let discord_webhook = env::var_os("DISCORD_WEBHOOK_URL");
        println!("{:?}", discord_webhook);
        if let Some(dwebhook) = discord_webhook {
            let mut builder = DiscordMessage::builder(dwebhook.to_str().unwrap());
            builder.gif_path("./t.jpg");
            builder.add_message("amessage");
            let dhm = builder.build();
            
            let rt = tokio::runtime::Runtime::new().expect("Could not create tokio runtime");
            rt.block_on(dhm.send()).unwrap();
        }
    }
}