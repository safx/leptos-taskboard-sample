all: client server

.PHONY: trunk
trunk:
	trunk build --filehash=false --release

.PHONY: client
client:
	wasm-pack build --target=web --features=hydrate --release

.PHONY: server
server:
	cargo build --bin server --features=ssr --release
