# Memorust 🦀⚡

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)  
In-memory key-value store inspired by Redis, written in Rust with Tokio for asynchronous operations.

## Features

- **In-memory storage**: Fast access with HashMap-based backend.
- **Async API**: Built on Tokio for high concurrency.
- **Command support**: `SET`, `GET`, `DEL`, `EXPIRE`, etc. (WIP).
- **Persistence**: Optional snapshotting to disk (planned).
- **Safe & Fast**: Leverages Rust's ownership model for thread safety.

## Quick Start

```bash
git clone https://github.com/LigeronAhill/memorust.git
cd memorust
cargo run --release
```
