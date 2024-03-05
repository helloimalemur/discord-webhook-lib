# rust-discord-lib
Discord Webhook library - VERY Simple lib for sending webhooks from your codebase, without fuss, no fluff


```rust
    let result: Result<(), reqwest::Error> = send_discord("webhook_url", "Hello World", "Lazarus").await
```
