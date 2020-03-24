# netstack

A batteries included networking crate for games.

## crates.io 📦

```
[dependencies]
netstack = "0.2.0"
```

## docks.rs 🗎

[Documentation (work in progress)](https://docs.rs/netstack/)

## requirements ❗

To effectively use netstack in production you need a secure channel to exchange an initial secret and connection token. It is recommended to use https for this, but it's up to you. The examples use a http server.

## features ☑️

(☑️ means implemented, 🚧 means planned or under development)

- ☑️ UDP Transport
- ☑️ Connection Management (connecting, heartbeats, timeouts, disconnects 🚧)
- ☑️ Packet Signing (HMAC SHA256)
- ☑️ Packet Acknowledgement (sequence numbers, acks, replay protection)
- 🚧 Derive Macro for easy binary serialization
- 🚧 Monitoring
- 🚧 Buffer Management
- 🚧 Switch between packet signing (bigger packet header) and encryption (more cpu hungry)

## non-goals ❌

### Event based I/O

Transports based on `io_uring`, `IOCP` or `epoll` are out of scope for me right now.

### Reliable Transmission

This crate does not implement retransmission based on acks and timeouts. Games have other ways of achieving reliability, mostly because information is already outdated by the time any timeout based mechanism would detect the lost packet.

FPS for instance send player input for the last couple of frames with every packet, so when one gets lost on the wire the next packet fills in the gap.

Compression of game state from the server to the client is usually based on the last packet acknowledged by the client. Use the `MessageAcknowledged` event and the sequence number returned by `send` for this.

## examples 🔌

See the [examples](/examples) directory for a client/server example, use the commands to run them:

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
