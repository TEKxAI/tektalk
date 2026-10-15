# Real MVP screen recordings

The Visual smoke workflow produces continuous MP4 videos, not a slideshow or
concept render, for Web, Android, iOS, macOS and Windows. Each flow creates an
account through its UI, logs out, logs in, answers the new-device challenge,
sends a message, and loads a real reply from the peer account. The peer is an
automated test account; its API checks prove the outgoing message is persisted
and readable by the recipient. Apple UI tests perform the same peer check.

The recordings use the real Rust HTTP server and local PostgreSQL. Redis,
Scylla and the business services are outside these HTTP auth/chat routes; the
separate CI local-stack job continues to test the full deployment.

Run the workflow on a pull request, or dispatch it from GitHub Actions. The
`tektalk-mvp-screen-recordings` artifact contains one combined H.264 MP4, five
platform videos and sanitized evidence reports. A failed flow retains raw
video/logs for diagnosis and cannot produce a successful combined artifact.
Fixtures contain only ephemeral local test accounts and are never published.

The current developer MVP opens a conversation with conversation/recipient
IDs. The video enters those fields through the UI; it does not inject tokens,
messages or hidden localStorage state into the app. Search/contact discovery
is outside this recording flow. No OTP is used for registration or login.
