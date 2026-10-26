# Quiver

A bridge between [Twitch EventSub](https://dev.twitch.tv/docs/eventsub) and RabbitMQ.

Quiver connects to Twitch over the EventSub WebSocket transport, subscribes to the
events you configure, and republishes each one as a JSON message to a RabbitMQ
exchange so the rest of your infrastructure can react to stream activity without
talking to Twitch directly.

## How it works

1. Reads `config.toml` from the working directory.
2. Resolves the configured event types into the OAuth scopes they require.
3. Starts a tiny HTTP server on `127.0.0.1:5000/auth` and prints an authorization
   URL. Opening that URL and approving it sends the authorization code back to
   Quiver.
4. Exchanges the code for an access token, then fetches the authenticated user's
   profile (used as the moderator/chatter identity in subscription conditions).
5. Connects to RabbitMQ and opens a channel.
6. Connects to `wss://eventsub.wss.twitch.tv/ws`. On `session_welcome` it registers
   one EventSub subscription per configured event type against
   `https://api.twitch.tv/helix/eventsub/subscriptions`.
7. Each incoming notification is routed to its typed struct, logged to stdout, and
   published to the configured exchange.

## Prerequisites

- Rust (edition 2024 toolchain)
- A running RabbitMQ instance
- A [Twitch application](https://dev.twitch.tv/console/apps) with its OAuth redirect
  URL set to `http://localhost:5000/auth`

## Setup

```bash
cp config.example.toml config.toml
```

Edit `config.toml`, like this:
```toml
broadcaster_id = "000000000"

events = [
    "channel.chat.message",
]

[RabbitMQ]
rabbit_mq_url = "https://rabbitMQURL:36675"
exchange_name = "twitch"
routing_key = "events"

[AppKeys]
client_id = "VerySecretClientId"
client_secret = "EvenMoreSecretClientSecret"
```

| Key | Description |
| --- | --- |
| `broadcaster_id` | Numeric user ID of the streamer to watch. |
| `events` | Event types to subscribe to. See the table below. |
| `rabbit_mq_url` | AMQP connection URL. |
| `exchange_name` | Exchange to publish to. |
| `routing_key` | Routing key used for every published message. |
| `client_id` | Twitch app client ID. |
| `client_secret` | Twitch app client secret. |

`config.toml` is gitignored, so real credentials stay out of the repository.

Then run:

```bash
cargo run --release
```

On startup Quiver prints an authorization URL. Open it in a browser and approve the
requested scopes; the flow completes automatically once you're redirected.

## Supported events

| Event type | Version | Required scope | Payload notes |
| --- | --- | --- | --- |
| `channel.bits.use` | 1 | `bits:read` | Bits cheered, power-up info, message |
| `channel.chat.message` | 1 | `user:read:chat` | Chat message with parsed fragments |
| `channel.follow` | 2 | `moderator:read:followers` | Follower and `followed_at` timestamp |
| `channel.raid` | 1 | 1 | Source/target broadcaster and viewer count |
| `channel.subscribe` | 1 | `channel:read:subscriptions` | Tier and whether the sub is a gift |
| `channel.subscription.gift` | 1 | `channel:read:subscriptions` | Number of gifted subs and cumulative total |

Scopes are derived automatically from the `events` list. You only authorize what
you actually subscribe to.

## Message format

Messages are published as JSON using serde's enum representation.
The event type is the outer key, wrapping the `event` and `subscription` objects:

```json
{
  "ChatMessage": {
    "event": {
      "broadcaster_user_id": "704122424",
      "broadcaster_user_name": "streamer",
      "chatter_user_id": "123456",
      "chatter_user_name": "viewer",
      "message_id": "abc-123",
      "message": {
        "text": "hello world",
        "fragments": [
          {
            "type": "text",
            "text": "hello world",
            "cheermote": null,
            "emote": null,
            "gif": null,
            "mention": null
          }
        ]
      }
    },
    "subscription": {
      "id": "sub-id",
      "type": "channel.chat.message",
      "version": "1",
      "cost": 0,
      "created_at": "2026-01-01T00:00:00Z"
    }
  }
}
```

The outer key varies per event: `BitsUse`, `ChatMessage`, `Follow`, ...

Optional fields (`cheermote`, `emote`, `gif`, ...) are always present in the output, 
serialized as `null` when unset.

Only the fields declared in the Rust structs under `src/events/` are retained, 
any additional fields Twitch sends are dropped during deserialization.

## Notes and limitations

- **The exchange is not declared.** Publishing assumes `exchange_name` already
  exists. Create it before running (see the `TODO` in `src/main.rs`).
- **Tokens are not refreshed.** The refresh token is stored but no refresh path is
  implemented, so a long-running instance will stop working when the access token
  expires. Restart to re-authenticate.
- **No replay-attack protection yet.** Message timestamps are not validated against
  the current time.
- **No graceful shutdown.** Stopping the process kills the WebSocket and AMQP
  connections abruptly.
- The **access token and authorization code are printed to stdout** during
  authentication. Just be mindful of logs.
