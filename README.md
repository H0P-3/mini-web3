# mini-web3 

A mini Web3 blockchain node written in pure Rust.

## Features
-   Proof-of-Work blockchain with configurable difficulty
-   Ed25519 wallet (key generation, signing, verification)
-   Signed transactions with mempool
-   Mining with coinbase reward
-   REST/JSON HTTP node (Actix-Web)
-   Chain integrity validation

## Project layout
```
src/
  main.rs        – entry point + CLI demo
  blockchain.rs  – Block, Transaction, Blockchain
  wallet.rs      – Wallet (Ed25519), verify_signature
  api.rs         – Actix-Web HTTP handlers
```

## Run

```bash
cargo run
```

The node starts at **http://127.0.0.1:8080**.

## API

| Method | Path | Description |
|--------|------|-------------|
| GET | `/chain` | Full blockchain |
| GET | `/balance/:address` | Confirmed balance |
| GET | `/pending` | Mempool transactions |
| GET | `/validate` | Chain integrity check |
| GET | `/wallet/new` | Generate a new wallet |
| POST | `/transaction` | Submit a signed transaction |
| POST | `/mine` | Mine pending transactions |

### Example – create wallet
```bash
curl http://localhost:8080/wallet/new
```

### Example – submit a transaction
```bash
curl -X POST http://localhost:8080/transaction \
  -H "Content-Type: application/json" \
  -d '{
    "from": "<alice_address>",
    "to": "<bob_address>",
    "amount": 5.0,
    "public_key": "<alice_pubkey_hex>",
    "signature": "<hex_sig>"
  }'
```

### Example – mine
```bash
curl -X POST http://localhost:8080/mine \
  -H "Content-Type: application/json" \
  -d '{"miner_address": "<your_address>"}'
```

## Push to GitHub

```bash
git init
git add .
git commit -m "feat: initial mini-web3 Rust node"
git branch -M main
git remote add origin https://github.com/<your-username>/mini-web3.git
git push -u origin main
```
