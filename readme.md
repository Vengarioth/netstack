# netstack

A batteries included netcode crate for games.

## examples

See the `example` directory for a client/server example.

## netstack_derive

While you can derive the `Deserialize` and `Serialize` trait yourself, netstack comes with its own procedural macro for convenience.

```rust
#[macro_use]
extern crate netstack_derive;

#[derive(Debug, Serialize, Deserialize)]
pub struct Greeting {
    pub id: u32,
    pub to: String,
    pub message: String,
}
```

## license

MIT
