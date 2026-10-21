# TEKtalk Web

The Web client is a zero-build HTML, CSS and JavaScript host for the Message, AI and Me experiences. It shares the backend contracts, authentication/session model, UUID client-message identifiers and Snowflake server-message identifiers used by the native clients. The Rust server serves these static assets in the complete local stack.

## Complete local demo

From the repository root:

```bash
make local-up
```

Open <http://localhost:8080/>. The initial screen is the standalone sign-in / registration view, with the same brand panel and form structure as Windows and macOS. The application stays hidden and inert until authentication (including new-device verification) succeeds. There is no demo bypass or preloaded chat data.

After signing in, desktop uses a 320px history sidebar with Message / AI / Me pinned to its bottom and one detail pane. Chat has inline conversation fields, a scrollable message area and a full-width composer footer. AI shows the plugin placeholder; Me shows account/device details, appearance controls and sign-out. A cached session is verified with an authenticated backend read before the application appears. Failed verification or an authenticated HTTP 401 returns to sign-in. Sign-out clears account previews and drafts.

## Frontend-only development

Start the backend separately, then run:

```bash
make web-dev
```

Open <http://localhost:5173/>. The development server calls `http://localhost:8080` by default. Override it with `?api=https://your-api.example`.

No npm installation or frontend build is required. Validate the client with `make web-check`.
