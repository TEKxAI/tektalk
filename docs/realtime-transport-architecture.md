# TEKtalk realtime transport architecture

## Decision

Native TEKtalk clients use a persistent TCP socket carrying MTProto 2.0 as the
primary Chat data plane. Secure WebSocket carries the identical MTProto
transport frames and is the mandatory fallback. HTTPS remains the control
plane for authentication, session bootstrap, DC discovery, configuration and
media upload negotiation.

Transport selection is owned by the shared Rust Core. Chat, sync and plugin
logic must not know whether a frame travelled over TCP or WSS.

## Connection order

1. Fetch a short-lived, single-use realtime ticket and signed DC endpoint list
   over HTTPS.
2. Attempt native TCP on port 443 with the MTProto abridged transport.
3. If TCP cannot connect or complete the protocol handshake within the
   configured deadline, connect to WSS on port 443.
4. Preserve the MTProto authorization key and logical session across transport
   changes. Create a new transport connection, not a new user login.
5. QUIC may be introduced later as an independently measured transport; it is
   not in the initial automatic selection order.

| Priority | Transport | Endpoint example | Role |
|---:|---|---|---|
| 1 | TCP + MTProto 2.0 | tcp://dc1.example:443 | native default |
| 2 | WSS + MTProto 2.0 | wss://dc1.example/v1/realtime | firewall/proxy fallback |
| control | HTTPS | https://api.example/v1/realtime/bootstrap | auth and discovery |

## Rust Core boundary

TransportManager exposes one ordered, reliable byte-stream abstraction:

- connect(endpoint_set)
- send(frame)
- receive()
- close(reason)
- connection-state and telemetry events

The TCP and WSS adapters implement the same trait. Above them, a single MTProto
codec owns framing, encryption, message IDs, sequence numbers, salts,
containers, acknowledgements, resend and replay rejection. Unsent and
unacknowledged messages stay in the logical session queue when the adapter is
replaced.

~~~mermaid
flowchart TB
    Host["Native host / plugin broker"] --> Core["Rust Chat Core"]
    Core --> Session["MTProto session + retry queue"]
    Session --> Manager["Transport Manager"]
    Manager --> TCP["TCP adapter :443"]
    Manager --> WSS["WSS adapter :443"]
    TCP --> Gateway["Realtime Gateway"]
    WSS --> Gateway
~~~

## Fallback policy

TCP is considered unavailable after DNS failure, connection refusal, network
policy rejection, handshake failure, or a connection/handshake deadline. A
protocol-authentication or key-validation failure is not eligible for silent
fallback; it terminates the attempt and raises a security event.

Fallback state is cached per network fingerprint for a bounded period so a
client behind a restrictive enterprise proxy does not retry TCP on every
reconnect. A network change clears that preference and starts again with TCP.
Exponential backoff uses jitter and a server-provided retry floor.

~~~mermaid
stateDiagram-v2
    [*] --> Bootstrap
    Bootstrap --> TcpConnecting
    TcpConnecting --> OnlineTcp: connected and authenticated
    TcpConnecting --> WssConnecting: transport unavailable
    WssConnecting --> OnlineWss: connected and authenticated
    OnlineTcp --> Reconnecting: connection lost
    OnlineWss --> Reconnecting: connection lost
    Reconnecting --> TcpConnecting: network changed
    Reconnecting --> WssConnecting: cached TCP block
~~~

## Gateway design

Both listeners terminate in the same gateway pipeline:

1. TCP L4 listener or WSS HTTP upgrade.
2. Connection admission, IP/device rate limits and ticket consumption.
3. MTProto transport-frame decoder with strict maximum lengths.
4. MTProto 2.0 authentication and encrypted-envelope validation.
5. Session routing and ordered delivery to Chat/Sync services over gRPC.

TCP and WSS gateways may scale independently but share session-routing state.
Deployments drain connections before shutdown and clients reconnect using the
same logical session.

## Safety requirements

- Never expose an unencrypted application payload on the TCP connection.
- Validate frame length before allocation and apply read/write/idle deadlines.
- Bind the one-use ticket to account, device, DC and a short expiration.
- Treat authentication, fingerprint and msg_key failures as security failures
  rather than network fallback signals.
- Apply per-IP, per-device and per-session connection and byte-rate limits.
- Do not give Valdi plugins or mini-apps direct access to sockets or auth keys.

## Observability and rollout

Measure connection success, handshake latency, fallback reason, reconnect
rate, bytes per message, ACK latency, delivery latency and battery/network
impact by platform and network type. Never log auth keys, decrypted payloads or
realtime tickets.

Roll out TCP through deterministic cohorts: internal, 1%, 5%, 25%, 50% and
100%. Automatically return a cohort to WSS when TCP connection success,
delivery SLO or crash-free sessions regress. WSS remains supported even after
TCP reaches 100% because it is the compatibility path for restrictive networks.
