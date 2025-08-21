use reqwest::{
    header,
    multipart::{Form, Part},
    Client,
};
use std::io;
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;

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
            extra_fields: None,
        }
    }

    pub async fn send(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Pre-read file (if any) and compute metadata so we can rebuild the form across retries
        let (file_bytes, filename, mime_opt): (Option<Vec<u8>>, Option<String>, Option<String>) =
            if let Some(image_path) = self.image_path.as_ref() {
                let bytes = tokio::fs::read(image_path)
                    .await
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, format!(
                        "Failed to read image file at '{}': {}",
                        image_path, e
                    )))?;
                let filename = Path::new(image_path)
                    .file_name()
                    .map(|os_str| os_str.to_string_lossy().to_string())
                    .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "Invalid file name"))?;
                let mime = guess_mime_from_filename(&filename).to_string();
                (Some(bytes), Some(filename), Some(mime))
            } else {
                (None, None, None)
            };

        let max_attempts: usize = 5;
        let mut last_status: Option<reqwest::StatusCode> = None;
        let mut last_body_snippet: Option<String> = None;

        for attempt in 0..max_attempts {
            // Build multipart form for this attempt
            let mut form = Form::new();

            if let (Some(ref bytes), Some(ref name), Some(ref mime)) = (&file_bytes, &filename, &mime_opt) {
                let part = Part::bytes(bytes.clone())
                    .file_name(name.clone())
                    .mime_str(mime)?;
                form = form.part("file1", part);
            }

            if let Some(ref fields) = self.extra_fields {
                for (key, value) in fields {
                    form = form.text(key.clone(), value.clone());
                }
            }

            if let Some(ref message) = self.message {
                form = form.text("content", message.clone());
            }

            let resp_result = self
                .client
                .post(&self.webhook_url)
                .multipart(form)
                .timeout(Duration::from_secs(30))
                .send()
                .await;

            match resp_result {
                Ok(res) => {
                    if res.status().is_success() {
                        return Ok(());
                    }

                    let status = res.status();
                    last_status = Some(status);

                    // Handle rate limiting (429)
                    if status.as_u16() == 429 {
                        let delay = if let Some(secs) = parse_retry_after_secs(res.headers()) {
                            // Convert seconds (may be fractional) to duration
                            let ms = (secs * 1000.0).ceil() as u64;
                            Duration::from_millis(ms)
                        } else {
                            compute_backoff(attempt as u32)
                        };

                        if attempt + 1 < max_attempts {
                            sleep(delay).await;
                            continue;
                        } else {
                            let body_text = res.text().await.ok();
                            last_body_snippet = body_text.map(|s| truncate(&s, 500));
                            break;
                        }
                    }

                    // Retry on server errors
                    if status.is_server_error() {
                        if attempt + 1 < max_attempts {
                            sleep(compute_backoff(attempt as u32)).await;
                            continue;
                        } else {
                            let body_text = res.text().await.ok();
                            last_body_snippet = body_text.map(|s| truncate(&s, 500));
                            break;
                        }
                    }

                    // For other 4xx, treat as fatal
                    let body_text = res.text().await.ok();
                    let snippet = body_text.as_deref().unwrap_or("");
                    return Err(Box::new(io::Error::new(
                        io::ErrorKind::Other,
                        format!(
                            "Discord webhook returned {}: {}",
                            status,
                            truncate(snippet, 500)
                        ),
                    )));
                }
                Err(err) => {
                    if attempt + 1 < max_attempts {
                        sleep(compute_backoff(attempt as u32)).await;
                        continue;
                    } else {
                        return Err(Box::new(io::Error::new(
                            io::ErrorKind::Other,
                            format!(
                                "Failed to send request after {} attempts: {}",
                                max_attempts, err
                            ),
                        )));
                    }
                }
            }
        }

        let status_str = last_status
            .map(|s| s.to_string())
            .unwrap_or_else(|| "unknown".to_string());
        let body_snip = last_body_snippet.unwrap_or_default();
        Err(Box::new(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "unable to send after retries (last status {}): {}",
                status_str, body_snip
            ),
        )))
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

fn guess_mime_from_filename(filename: &str) -> &'static str {
    let ext = Path::new(filename)
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_ascii_lowercase());
    match ext.as_deref() {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("bmp") => "image/bmp",
        Some("tiff") | Some("tif") => "image/tiff",
        Some("svg") => "image/svg+xml",
        _ => "application/octet-stream",
    }
}

fn parse_retry_after_secs(headers: &header::HeaderMap) -> Option<f64> {
    if let Some(v) = headers.get(header::RETRY_AFTER) {
        if let Ok(s) = v.to_str() {
            if let Ok(n) = s.parse::<f64>() {
                return Some(n.max(0.0));
            }
        }
    }
    if let Some(v) = headers.get("x-ratelimit-reset-after") {
        if let Ok(s) = v.to_str() {
            if let Ok(n) = s.parse::<f64>() {
                return Some(n.max(0.0));
            }
        }
    }
    None
}

fn compute_backoff(attempt: u32) -> Duration {
    const BASE_MS: u64 = 500;
    const MAX_MS: u64 = 5_000;
    let shift = attempt.min(63);
    let factor = 1u64 << shift;
    let ms = BASE_MS.saturating_mul(factor).min(MAX_MS);
    Duration::from_millis(ms)
}

fn truncate(s: &str, max: usize) -> String {
    let mut out: String = s.chars().take(max).collect();
    if s.chars().count() > max {
        out.push_str("...");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
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

    #[test]
    fn test_guess_mime_from_filename() {
        assert_eq!(guess_mime_from_filename("image.jpg"), "image/jpeg");
        assert_eq!(guess_mime_from_filename("image.JPEG"), "image/jpeg");
        assert_eq!(guess_mime_from_filename("image.png"), "image/png");
        assert_eq!(guess_mime_from_filename("image.gif"), "image/gif");
        assert_eq!(guess_mime_from_filename("image.webp"), "image/webp");
        assert_eq!(guess_mime_from_filename("image.unknown"), "application/octet-stream");
    }

    #[test]
    fn test_compute_backoff() {
        assert_eq!(compute_backoff(0), Duration::from_millis(500));
        assert_eq!(compute_backoff(1), Duration::from_millis(1_000));
        assert_eq!(compute_backoff(2), Duration::from_millis(2_000));
        assert_eq!(compute_backoff(3), Duration::from_millis(4_000));
        assert_eq!(compute_backoff(4), Duration::from_millis(5_000)); // capped
        assert_eq!(compute_backoff(10), Duration::from_millis(5_000)); // still capped
    }
}