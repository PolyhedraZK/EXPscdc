build:
	cargo build

release:
	cargo build --release

fmt:
	cargo fmt
run:
	cargo run -- --path ./data --url http://127.0.0.1:26657
