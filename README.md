# Kraken REST Adapter (Rust)

This repository exposes a compact Axum-based REST API that proxies the
Kraken private order endpoints. It exists so higher-level strategy
services can submit, cancel, and inspect orders without worrying about
Kraken's signing requirements.

## Requirements

- Rust 1.76+
- Kraken API key/secret with trading permissions

## Setup

1. Copy your Kraken credentials into environment variables (a `.env` file
   loaded via `dotenvy` also works):

   ```bash
   export KRAKEN_API_KEY="your_key"
   export KRAKEN_API_SECRET="your_secret"
   # optional override for sandboxes
   # export KRAKEN_API_BASE_URL="https://api.kraken.com"
   ```

2. Build and run the server:

   ```bash
   cargo run
   ```

   Override the listen port with `PORT=9000 cargo run` if needed.

## API Surface

| Method | Path             | Kraken Endpoint | Description                    |
| ------ | ---------------- | --------------- | ------------------------------ |
| POST   | `/orders`        | `AddOrder`      | Submit a new order             |
| GET    | `/orders/{txid}` | `QueryOrders`   | Fetch order status/metadata    |
| DELETE | `/orders/{txid}` | `CancelOrder`   | Cancel an existing order       |

Payloads are modeled with Rust enums/structs (see `models.rs`) so you get
compile-time validation for order sides, order types, and time-in-force.
Unknown Kraken fields can still be passed through by supplying
`extra` key/value pairs in the `POST /orders` body.

The service returns the typed Kraken responses directly, making it easy
for downstream components to reason about state transitions.
