# cody

### Installation
```shell
cargo install dioxus-cli
```

### Development
Run with hot reloading
```shell
dx serve --hot-patch --features local
```
Run wasm build (install trunk via `cargo binstall trunk`)
```shell
rustup target add wasm32-unknown-unknown
trunk serve
```

Run tests with
```shell
cargo test
```
