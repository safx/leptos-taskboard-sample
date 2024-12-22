all: workers

.PHONY: trunk
trunk:
	trunk build --filehash=false --features=csr --release

.PHONY: client
client:
	wasm-pack build --target=web --features=hydrate --release
	cp style.css pkg/

.PHONY: server
server: client
	cargo build --bin server --features=ssr --release

.PHONY: workers
workers: client
	npx wrangler dev

.PHONY: clean
clean:
	rm -fr build pkg target npm_modules
