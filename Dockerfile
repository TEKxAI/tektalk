FROM rust:1.98-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.toml
COPY server server
COPY infra infra
RUN cargo build --release -p chat-server
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/chat-server /usr/local/bin/chat-server
USER 65532:65532
EXPOSE 8080
HEALTHCHECK --interval=10s --timeout=3s --start-period=10s --retries=12 CMD curl --fail --silent http://127.0.0.1:8080/healthz || exit 1
ENTRYPOINT ["chat-server"]
