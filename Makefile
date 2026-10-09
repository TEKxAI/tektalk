.PHONY: dev web-dev web-check test fmt lint infra-up infra-down local-up local-down local-reset smoke logs
dev:
	cargo run -p chat-server
web-dev:
	./scripts/web-dev.sh
web-check:
	./scripts/validate-web.sh
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
local-up:
	./scripts/local-up.sh
local-down:
	./scripts/local-down.sh
local-reset:
	./scripts/local-down.sh --volumes
smoke:
	./scripts/smoke-test.sh
logs:
	docker compose logs -f server
