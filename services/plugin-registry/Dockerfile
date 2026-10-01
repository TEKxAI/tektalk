FROM rust:1.98-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.toml
COPY core core
COPY server server
COPY services services
COPY contracts contracts
COPY infra infra
RUN cargo build --release -p tektalk-plugin-registry

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates netcat-openbsd && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/tektalk-plugin-registry /usr/local/bin/tektalk-plugin-registry
USER 65532:65532
EXPOSE 50051
HEALTHCHECK --interval=10s --timeout=3s --start-period=5s --retries=12 CMD nc -z 127.0.0.1 50051 || exit 1
ENTRYPOINT ["tektalk-plugin-registry"]
