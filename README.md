# leptos-taskboard-sample

Execute the following commands for CSR app:
```bash
rustup target add wasm32-unknown-unknown
cargo install --locked trunk
cargo install --locked wasm-bindgen-cli
git clone https://github.com/safx/leptos-taskboard-sample
cd leptos-taskboard-sample
trunk serve --filehash=false --features=csr --release --open
```

Additionally, execute the following commands for SSR + Hydration app:
```bash
cargo install wasm-pack
wasm-pack build --target=web --features=hydrate --release
cargo run --bin server --features=ssr --release
```
