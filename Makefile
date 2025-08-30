run:
	@set -a; \
	if [ -f .env ]; then source .env; fi; \
	set +a; \
	cargo run

build:
	cargo build

release:
	cargo build --release

test:
	cargo test
