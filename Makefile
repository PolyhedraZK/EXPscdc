build:
	cargo build

release:
	cargo build --release
install: release
	rm -f ~/.cargo/bin/scd
	cp target/release/scd ~/.cargo/bin/
fmt:
	cargo fmt
run:
	cargo run -- --path ./data --url http://127.0.0.1:26657
