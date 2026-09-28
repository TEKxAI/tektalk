FROM rust:1.98-bookworm AS build
WORKDIR /src

COPY Cargo.toml Cargo.toml
COPY server server
COPY services services
COPY contracts contracts
RUN cargo build --release -p tektalk-platform-services --bins

FROM debian:bookworm-slim
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates netcat-openbsd \
    && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/signin-signup /usr/local/bin/signin-signup
COPY --from=build /src/target/release/chat /usr/local/bin/chat
COPY --from=build /src/target/release/session-management /usr/local/bin/session-management
COPY --from=build /src/target/release/consent-management /usr/local/bin/consent-management
USER 65532:65532
