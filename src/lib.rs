use anyhow::Error;
use reqwest::{multipart::{Form, Part}, Client};
use std::fs;
use std::path::Path;

pub struct DiscordMessage {
    client: Client,
    webhook_url: String,
    image_path: Option<String>,
    message: Option<String>,
    extra_fields: Option<Vec<(String, String)>>,
}

impl DiscordMessage {
    pub fn builder(webhook_url: impl Into<String>) -> DiscordMessageBuilder {
        DiscordMessageBuilder {
            webhook_url: webhook_url.into(),
            gif_path: None,
            message: None,
            extra_fields: None
        }
    }

    pub async fn send(&self) -> Result<(), Box<dyn std::error::Error>> {

        let mut part = Part::text("");
        
        if let Some(image_path) = self.image_path.as_ref() {
            let file_bytes = fs::read(image_path)?;
            let filename = Path::new(image_path)
                .file_name()
                .map(|os_str| os_str.to_string_lossy().to_string())
                .ok_or("Invalid file name")?;

            part = Part::bytes(file_bytes)
                .file_name(filename.clone())
                .mime_str("image/gif")?;
        }

        let mut form = Form::new().part("file1", part);
        if let Some(ref fields) = self.extra_fields {
            for (key, value) in fields {
                form = form.text(key.clone(), value.clone());
            }
        }

        if let Some(ref message) = self.message {
            form = form.text("content", message.clone());
        }

        let res = self.client
            .post(&self.webhook_url)
            .multipart(form)
            .send()
            .await?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(Box::from(Error::msg("unable to send")))
        }
    }
}

pub struct DiscordMessageBuilder {
    webhook_url: String,
    gif_path: Option<String>,
    message: Option<String>,
    extra_fields: Option<Vec<(String, String)>>,
}

impl DiscordMessageBuilder {
    pub fn gif_path(&mut self, path: impl Into<String>) {
        self.gif_path = Some(path.into());
    }

    pub fn add_message(&mut self, message: impl Into<String>) {
        self.message = Option::from(message.into());
    }

    pub fn add_field(&mut self, key: impl Into<String>, value: impl Into<String>) {
        match self.extra_fields {
            Some(ref mut fields) => fields.push((key.into(), value.into())),
            None => self.extra_fields = Some(vec![(key.into(), value.into())]),
        }
    }

    pub fn build(self) -> DiscordMessage {
        DiscordMessage {
            client: Client::new(),
            webhook_url: self.webhook_url,
            image_path: self.gif_path,
            message: self.message,
            extra_fields: self.extra_fields,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::DiscordMessage;
    use std::env;

    #[test]
    fn send_test() {
        let discord_webhook = env::var_os("DISCORD_WEBHOOK_URL");
        println!("{:?}", discord_webhook);
        if let Some(dwebhook) = discord_webhook {
            let mut builder = DiscordMessage::builder(dwebhook.to_str().unwrap());
            builder.gif_path("./t.jpg");
            // builder.add_message("amessage");
            builder.add_field("username", "test");
            builder.add_field("content", "filename");
            let dhm = builder.build();

            let rt = tokio::runtime::Runtime::new().expect("Could not create tokio runtime");
            rt.block_on(dhm.send()).unwrap();
        }
    }
    #[test]
    fn send_message_only() {
        let discord_webhook = env::var_os("DISCORD_WEBHOOK_URL");
        println!("{:?}", discord_webhook);
        if let Some(dwebhook) = discord_webhook {
            let mut builder = DiscordMessage::builder(dwebhook.to_str().unwrap());
            builder.add_field("username", "test");
            builder.add_field("content", "a message");
            let dhm = builder.build();

            let rt = tokio::runtime::Runtime::new().expect("Could not create tokio runtime");
            rt.block_on(dhm.send()).unwrap();
        }
    }
    
}