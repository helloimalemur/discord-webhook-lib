use reqwest::{Client, multipart::{Form, Part}};
use std::path::Path;
use std::fs;
use mime;

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
            extra_fields: None,
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
    extra_fields: Option<Vec<(String, String)>>,
    message: Option<String>
}

impl DiscordMessageBuilder {
    /// Set the path to the GIF file.
    pub fn gif_path(&mut self, path: impl Into<String>) {
        self.gif_path = Some(path.into());
    }

    /// Add an extra field (key-value pair) to the multipart form.
    pub fn add_field(&mut self, key: impl Into<String>, value: impl Into<String>) {
        match self.extra_fields {
            Some(ref mut fields) => fields.push((key.into(), value.into())),
            None => self.extra_fields = Some(vec![(key.into(), value.into())]),
        }
    }
    // let builder = DiscordMessageBuilder::new();
    
    // let a_new_builder = builder.add_field("asdf", "asdf");
    // builder.add_field("asdf", "asdf");

    /// Add an extra field (key-value pair) to the multipart form.
    pub fn add_message(&mut self, message: impl Into<String>) {
        self.message = Option::from(message.into());
    }

    /// Finalizes the builder and creates a DiscordMessage instance.
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
    use std::env;
    use std::env::args_os;
    use crate::message::{DiscordMessage, DiscordMessageBuilder};

    #[test]
    fn send_test() {
        let discord_webhook = env::var_os("DISCORD_WEBHOOK_URL");
        if let Some(dwebhook) = discord_webhook {
            let mut builder = DiscordMessage::builder(dwebhook.to_str().unwrap());
            builder.gif_path("/gifs/gif.png");
            builder.add_message("amessage");
            let dhm = builder.build();
        }
    }
}