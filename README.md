# cody
Cody is an agentic coding ide built for performance using [egui](https://github.com/emilk/egui) and [wgpu](https://github.com/gfx-rs/wgpu). Its heavily inspired by GUI tools like Cursor and Conductor as well as CLI tools like opencode and pi.


### Installation
You'll need the dioxus cli in order to run the app with hot reloading
```shell
cargo install dioxus-cli
```


### Commands

Run the app (hot reloading)
```shell
dx serve --hot-patch --features local
```

Run tests
```shell
cargo test
```

Run linting
```shell
cargo clippy
```

Run formatting
```shell
cargo fmt
```
