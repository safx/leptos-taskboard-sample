all: client workers

.PHONY: trunk
trunk:
	trunk build --filehash=false --features=csr --release

.PHONY: client
client:
	wasm-pack build --target=web --features=hydrate --release
	cp style.css pkg

.PHONY: server
server:
	cargo build --bin server --features=ssr --release

.PHONY: workers
workers:
	bun x wrangler dev

.PHONY: clean
clean:
	rm -fr build pkg target npm_modules
