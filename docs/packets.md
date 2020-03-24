# Packet

## Data Layout

| Field               | Type       | Size |
|---------------------|------------|------|
| HMAC                | `[u8; 32]` | 32   |
| Sequence Number     | `u64`      | 8    |
| Ack Sequence Number | `u64`      | 8    |
| Ack Bits            | `[u8; 4]`  | 4    |
| Packet Type         | `u8`       | 1    |
| Padding (free)      | `u8`       | 1    |
| Body Length         | `u16`      | 2    |
| Body                | `[u8; ?]`  | ?    |

### HMAC

The [HMAC](https://en.wikipedia.org/wiki/HMAC) is the cryptographic signature of the packet, derived from the contents of the packet, including the header but without the hmac itself, and the `session secret`. Any modification to the signature or the packet after signing invalidates the packets cryptographic integrity. This protects against malicious modification or packet corruption.

### Sequence Number

The sequence number is used to perform acknowledgements and also acts as a [nonce](https://en.wikipedia.org/wiki/Cryptographic_nonce) against replay attacks and bad networks. Each sequence number is processed one time at most.

### Ack Sequence Number

The highest known sequence number **+ 1** the sending party received from the remote party. This is used to acknowledge messages. This will change in the future with a more elegant implementation.

### Ack Bits

This field is used to acknowledge the following 32 sequence numbers after the `Ack Sequence Number` `S`: Each bit `b` represents the Sequence Number `S - b`.

### Packet Type

A number specifying the type of the packet as follows:

| Number | Packet Type         |
|--------|---------------------|
| 0      | Connection Packet   |
| 1      | Payload Packet      |
| 2      | Heartbeat Packet    |
| 3      | Disconnect Packet   |
| 4      | Disconnected Packet |

### Body Length

This field contains the length of the body. The body of the packet must have exactly this size. The maximum size is implementation dependant, but will never exceed common [MTU](https://en.wikipedia.org/wiki/Maximum_transmission_unit) values.

### Body

The body contains the packet payload data, if any.
