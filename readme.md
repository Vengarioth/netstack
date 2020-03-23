# netstack

A batteries included networking crate for games.

## crates.io 📦

```
[dependencies]
netstack = "0.1.1"
```

## requirements ❗

To effectively use netstack in production you need a secure channel to exchange an initial secret and connection token. It is recommended to use https for this, but it's up to you. The examples use a http server.

## features ☑️

- ☑️ UDP Transport
- ☑️ Connection Management (connecting, heartbeats, timeouts, 🚧 disconnects)
- ☑️ Packet Signing (HMAC SHA256)
- 🚧 Packet Acknowledgement (sequence numbers, acks, replay protection)
- 🚧 Derive Macro for easy binary serialization
- 🚧 Monitoring

## examples 🔌

See the `example` directory for a client/server example, use the commands to run them:
* server: `cargo run -p server`
* client: `cargo run -p client`

## netstack_derive 🚧

Netstack comes with a _work in progress_ derive macro for structs (and later enums).

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Greeting {
    pub id: u32,
    pub to: String,
    pub message: String,
}
```

## license 📃

[MIT](/LICENSE)
