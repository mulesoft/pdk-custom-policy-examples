# WebSocket Frame Counter Policy Example

This policy intercepts WebSocket connections and prefixes each upstream text
frame with a sequential counter. Sending `"hello"` returns `"1:hello"`;
`"world"` returns `"2:world"`. Binary and control frames (Ping, Pong, Close)
pass through unchanged.

Use this example as a starting point for any policy that needs to inspect or
transform WebSocket frames using the PDK high-level framework.

To learn more about WebSocket support in PDK, see the
[PDK WebSocket documentation](https://docs.mulesoft.com/pdk/latest/policies-pdk-configure-features-websocket).

## What this example demonstrates

| PDK concept | Where used |
|---|---|
| `on_create` | Initializes per-connection `CounterState` |
| `on_upgrade_upstream` | Intercepts client→server frames, prefixes text |
| `on_upgrade_downstream` | Passes server→client frames unchanged |
| `on_done` | Logs total frames when the WebSocket session ends |
| `Decoder::parse` | Accumulates TCP chunks into complete frames |
| `Encoder::encode_client` | Re-encodes transformed frames |
| Shared state | `CounterState` is the same instance across all handlers |

## How it works

1. When a WebSocket upgrade is detected, `on_create` creates a `CounterState`
   with `frame_count = 0`.
2. For every upstream text frame the counter increments and the payload is
   prefixed: `"{n}:{original}"`.
3. Non-text frames and downstream frames pass through without modification.
4. When the connection closes, `on_done` logs how many frames were counted.

## Policy configuration

This policy has no configuration parameters — it works out of the box.

## Test the policy

Test using unit tests or the policy playground.

### Unit tests

```shell
make test
```

The test suite covers:

- Text frames are prefixed with an incrementing counter.
- Binary frames are not counted and pass through unchanged.
- Control frames (Ping, Pong) pass through unchanged.
- Counter is not incremented by binary or control frames.
- Incomplete (fragmented) frames are accumulated before processing.

### Playground testing

1. Build the policy:

```shell
make build
```

2. Place a `registration.yaml` file in `playground/config/` to connect to
   Anypoint Platform.

3. Start Flex Gateway and the echo server:

```shell
make run
```

Flex listens on `localhost:8081`. The echo server reflects every frame back
to the client, so each message you send will be returned with the counter
prefix added by the policy.

4. Connect with a WebSocket client such as `websocat`:

```shell
websocat ws://localhost:8081/
```

5. Type messages and observe the counter prefix:

```
hello        # you send
1:hello      # you receive (echoed back with prefix)
world        # you send
2:world      # you receive
```