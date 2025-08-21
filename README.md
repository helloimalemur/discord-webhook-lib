# rust-discord-lib
Discord Webhook library - VERY Simple lib for sending webhooks from your codebase, without fuss, no fluff



```rust
let mut builder = DiscordMessage::builder(webhook_url.clone());
builder.add_message(full_msg);
let dmb = builder.build();

if let Err(e) = dmb.send().await
{
println!("{e}")
}
```


```rust
let mut builder = DiscordMessage::builder(webhook_url);
builder.add_field("username", "Lazarus");
builder.add_field("content", "a message");
let dhm = builder.build();
let result: Result<(), reqwest::Error> = dhm.send();
```

## Reliability improvements (no API changes)
- Automatic retries with exponential backoff on transient failures (network errors and 5xx) and Discord rate limits (HTTP 429). When rate limited, the library respects the Retry-After and X-RateLimit-Reset-After headers.
- Improved error messages: non-success responses include HTTP status and a short snippet of the response body.
- Async, non-blocking file reads and only attaches a file part when a file path is provided.

Your existing code continues to work exactly the same; the public API is unchanged.

### Example (message only)
```rust
let mut builder = DiscordMessage::builder(webhook_url);
builder.add_field("username", "Lazarus");
builder.add_field("content", "a message");
let dhm = builder.build();

// In an async context
if let Err(e) = dhm.send().await {
    eprintln!("{e}");
}

// From a sync context
let rt = tokio::runtime::Runtime::new().unwrap();
rt.block_on(async {
    if let Err(e) = dhm.send().await {
        eprintln!("{e}");
    }
});
```

### Tests
- Offline unit tests are included for helper logic (MIME detection and backoff computation).
- Integration-like tests will only execute the network call if the DISCORD_WEBHOOK_URL environment variable is set.
