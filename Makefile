.PHONY: dev test fmt lint infra-up infra-down
dev:
	cargo run -p chat-server
test:
	cargo test --workspace
fmt:
	cargo fmt --all
lint:
	cargo clippy --workspace --all-targets -- -D warnings
infra-up:
	docker compose up -d postgres redis scylla redpanda
infra-down:
	docker compose down
