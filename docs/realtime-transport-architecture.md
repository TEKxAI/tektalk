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
5. QUIC is negotiated independently for file/media transfer. It does not
   replace the ordered TCP/WSS selection for MTProto chat/control messages.

| Priority | Transport | Endpoint example | Role |
|---:|---|---|---|
| 1 | TCP + MTProto 2.0 | tcp://dc1.example:443 | native default |
| 2 | WSS + MTProto 2.0 | wss://dc1.example/v1/realtime | firewall/proxy fallback |
| media | QUIC Streams | quic://media-dc1.example:443 | encrypted file chunks |
| realtime media | QUIC DATAGRAM | quic://media-dc1.example:443 | voice/video packets |
| control | HTTPS | https://api.example/v1/realtime/bootstrap | auth and discovery |

## TEKtalk Adaptive Transport

Adaptive Transport is a data-plane extension, not a new cryptographic or chat
protocol. MTProto 2.0 continues to own chat/control delivery and Secret Chat
continues to own end-to-end keys.

File and media transfer follows these rules:

1. The sender creates a signed transfer manifest containing the file identity,
   conversation, Secret Chat epoch, exact length, chunk geometry and SHA-256.
2. Rust Core derives a file-specific key from the Secret Chat session key with
   HKDF-SHA-256. Each chunk is independently protected with
   AES-256-GCM-SIV; the manifest identity, index and total count are bound as
   authenticated data.
3. Platform QUIC adapters open a bounded number of reliable streams. Core
   selects 1, 2, 4 or 8 streams from measured bandwidth, RTT and packet loss.
4. The receiver persists only the completed-chunk bitmap. Reconnects request
   missing indexes and never restart completed chunks.
5. The receiver verifies the complete plaintext length and SHA-256 after
   ordered reassembly.
6. Voice/video adapters use QUIC DATAGRAM with a Core-provided bitrate, FEC and
   retransmission-window profile. Late packets are dropped instead of blocking
   newer media.

The server sees ciphertext, chunk metadata and delivery state. One-time upload
and download capabilities must be bound to account, device, conversation,
file, expiry and operation. Delivered temporary chunks are removed after a
short retry grace period.

QUIC is enabled through a separately advertised capability and deterministic
cohorts. If UDP/QUIC is unavailable, encrypted file chunks use the existing
HTTPS upload path; calls retain the existing supported media path. A security
or authentication failure is never treated as a transport fallback condition.

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

Adaptive Transport additionally measures useful goodput, per-chunk retry rate,
resume success, time to first media, p50/p95 completion time, loss, jitter,
stream count and energy per transferred MiB. Promotion requires an A/B result
against the existing path; protocol overhead alone is not a speed claim.

Roll out TCP through deterministic cohorts: internal, 1%, 5%, 25%, 50% and
100%. Automatically return a cohort to WSS when TCP connection success,
delivery SLO or crash-free sessions regress. WSS remains supported even after
TCP reaches 100% because it is the compatibility path for restrictive networks.
