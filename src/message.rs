struct DiscordMessage {
    message: String,
    file: Option<String>,
}

impl DiscordMessage {
    pub fn add_message(&mut self, message: &str) {
        self.message.clone_from(&message.to_string());
    }
    pub fn add_file(&mut self, file: &str) {
        self.file = Some(file.to_string());
    }
    pub fn send_message(&mut self) {
        
    }
}

struct DiscordMessageBuilder {
    url: String,
    username: Option<String>,
}

impl DiscordMessageBuilder {
    pub fn new(url: String, username: String) -> DiscordMessageBuilder {
        DiscordMessageBuilder {
            url,
            username: Some(username),
        }
    }
    pub fn create_message(&self, message: String) -> DiscordMessage {
        DiscordMessage {
            message,
            file: None,
        }
    }
}