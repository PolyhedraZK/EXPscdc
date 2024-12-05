build:
	cargo build

release:
	cargo build --release

install: release
	rm -f ~/.cargo/bin/EXPscdc
	cp -f target/release/EXPscdc ~/.cargo/bin/

fmt:
	cargo fmt

run:
	cargo run --release -- --path ./data --url http://127.0.0.1:26657
