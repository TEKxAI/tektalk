# TEKtalk Web

The Web client is a zero-build HTML, CSS and JavaScript host for the Message, AI and Me experiences. It shares the backend contracts, authentication/session model, UUID client-message identifiers and Snowflake server-message identifiers used by the native clients. The Rust server serves these static assets in the complete local stack.

## Complete local demo

From the repository root:

```bash
make local-up
```

Open <http://localhost:8080/>. Register a new account to use the real local backend, or select **Explore demo mode** to inspect the interface without creating data.

## Frontend-only development

Start the backend separately, then run:

```bash
make web-dev
```

Open <http://localhost:5173/>. The development server calls `http://localhost:8080` by default. Override it with `?api=https://your-api.example`.

No npm installation or frontend build is required. Validate the client with `make web-check`.
