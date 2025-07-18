# rust-discord-lib
Discord Webhook library - VERY Simple lib for sending webhooks from your codebase, without fuss, no fluff


```rust
let mut builder = DiscordMessage::builder(webhook_url);
builder.add_field("username", "Lazarus");
// builder.add_field("content", "a message");
builder.add_message("a message");
let dhm = builder.build();
let result: Result<(), reqwest::Error> = dhm.send();


let mut builder = DiscordMessage::builder(webhook_url.clone());
builder.add_message(full_msg);
let dmb= builder.build();

if let Err(e) = dmb.send().await
{
println!("{e}")
}
```
