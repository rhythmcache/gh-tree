#!/bin/bash

curr=$(pwd)

mkdir -p "$curr/ghtree/src" "$curr/ghrls/src" && cp "$curr/ghrls.rs" "$curr/ghrls/src/main.rs" && cp "$curr/ghtree.rs" "$curr/ghtree/src/main.rs"

cat << 'EOF' > "$curr/ghtree/Cargo.toml"
[package]
name = "ghtree"
version = "0.4.0"
edition = "2021"

[dependencies]
clap = { version = "4.4", features = ["derive"] }
tokio = { version = "1.36", features = ["full"] }
# reqwest = { version = "0.12", features = ["json", "stream"] }
reqwest = { version = "0.12", features = ["json", "stream", "rustls-tls"], default-features = false }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
futures = "0.3"
anyhow = "1.0"
directories = "6.0"
indicatif = "0.17"
async-stream = "0.3"
colored = "3.0"
tokio-stream = "0.1"
EOF

cat << 'EOF' > "$curr/ghrls/Cargo.toml"
[package]
name = "ghrls"
version = "0.3.0"
edition = "2021"

[dependencies]
anyhow = "1.0"
bytes = "1.5"
clap = { version = "4.4", features = ["derive"] }
futures-util = "0.3"
humansize = "2.1"
indicatif = "0.17"
reqwest = { version = "0.12", features = ["json", "stream", "rustls-tls"], default-features = false }
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.35", features = ["full"] }
colored = "3.0"
EOF

if ! command -v cargo &>/dev/null; then
    echo "Error: Rust Not Installed?"
    echo "Please Install Rust"
    exit 1
fi
idk="$curr/ghtree/target/release/ghtree"
idk2="$curr/ghrls/target/release/ghrls"
cd "$curr/ghtree" && cargo build --release && cd "$curr/ghrls" && cargo build --release && echo "Build Completed!!!" && [ -f "$idk" ] && echo "ghtree is at $idk" && [ -f "$idk2" ] && echo "ghrls is at $idk2" && echo "Done" && exit 0
