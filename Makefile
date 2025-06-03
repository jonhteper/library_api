all: prepare
	cargo fmt
	cargo build --package library_api --bin library_api --release
	cp target/release/library_api packages/library-api
	cargo build --package library_api --bin library_cli --features cli --release
	cp target/release/library_cli packages/library-cli

prepare:
	mkdir packages || true
	touch packages/library_api.db
	touch packages/.env
	@echo "# Configuración por defecto de la API" > packages/.env
	@echo "DATABASE_URL=sqlite:library_api.db" >> packages/.env
	@echo "API_PORT=8080" >> packages/.env
	@echo "RUST_LOG=info" >> packages/.env

dev: prepare
	cargo fmt
	cargo build --package library_api --bin library_api
	cp target/debug/library_api packages/library-api

dev-cli: prepare
	cargo fmt
	cargo build --package library_api --bin library_cli --features cli
	cp target/debug/library_cli packages/library-cli




test:
	cargo test -- --show-output

all-test:
	cargo test --features integration-tests -- --show-output

lint:
	cargo clippy
