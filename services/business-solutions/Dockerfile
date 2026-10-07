FROM rust:1.98-bookworm AS build
WORKDIR /src
COPY Cargo.toml Cargo.toml
COPY core core
COPY server server
COPY services services
COPY contracts contracts
RUN cargo build --release -p tektalk-business-solutions

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates netcat-openbsd && rm -rf /var/lib/apt/lists/*
COPY --from=build /src/target/release/business-solutions /usr/local/bin/business-solutions
USER 65532:65532
ENTRYPOINT ["/usr/local/bin/business-solutions"]
