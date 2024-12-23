# leptos-taskboard-sample

## CSR app
```bash
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
cargo install --locked wasm-bindgen-cli
git clone https://github.com/safx/leptos-taskboard-sample
cd leptos-taskboard-sample

trunk serve --filehash=false --features=csr --release --open
```

## SSR app
```bash
cargo install wasm-pack
wasm-pack build --target=web --features=hydrate --release
make server && ./target/release/server
```

## Cloudflare Workers (D1 + SSR)
```bash
make
```
