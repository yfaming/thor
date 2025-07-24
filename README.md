# ⚡ Thor – A Lightning Address Server Powered by Rust and NWC

**Your domain. Your address. No node. Just Rust and NWC.**

Thor is a fast, self-hosted Lightning Address server written in Rust.

By using NWC (Nostr Wallet Connect), it allows users to receive Lightning payments through their own domain — **without running a Lightning Network node**.


## ✨ Features

- ⚡ Custom Lightning addresses like `you@yourdomain.com`
- 🧠 No need to run LND or Core Lightning — just connect to your wallet via NWC
- 🦀 High-performance and safe — implemented in Rust
- 🧩 Self-hosted and easy to deploy
- 🔧 Configurable and extensible architecture (LND, CLN support planned)


## 🚀 Getting Started

```bash
# clone and build the code
git clone https://github.com/yfaming/thor
cd thor
cargo build

# edit the config file
mv config.toml.example config.toml
vim config.toml

# run!
cargo run -- ./config.toml
```

## Lightning address specs:
- [LUD-16: Paying to static internet identifiers](https://github.com/lnurl/luds/blob/luds/16.md)
- [LUD-06: payRequest base spec](https://github.com/lnurl/luds/blob/luds/06.md)
