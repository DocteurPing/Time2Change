# Time2Change

> **Should you exchange your money right now?** Time2Change answers that question by analysing historical exchange-rate data and telling you whether the current rate is near the top, bottom, or middle of its recent range.

---

## What it does

You pick a currency pair (e.g. EUR/USD) and a lookback window (up to 365 days). The app:

1. Loads the stored historical rates for that pair.
2. Finds where today's rate sits inside the observed min–max range.
3. Returns a plain-English recommendation — **Change Now**, **Wait**, or **Neutral** — along with a confidence score.

---

## Architecture

The project is a Rust workspace split into focused crates, following a clean/hexagonal layering:

| Crate | Role |
|---|---|
| `domain` | Pure business logic — currency types, time series, rate quality scoring, math indicators |
| `application` | Use cases (`AnalyzePair`, `IngestRates`, `SyncCurrencies`) and port traits |
| `infrastructure` | PostgreSQL repositories and DB migrations |
| `api` | Axum HTTP server — exposes the REST endpoints |
| `ingestion` | Background service — polls external rate providers and writes to Postgres |
| `frontend` | Leptos/WebAssembly UI — the browser dashboard |
| `shared` | Cross-cutting utilities (tracing setup, etc.) |

---

## Tech stack

- **Rust** — entire stack, end to end
- **Tokio** — async runtime
- **Axum** — REST API
- **Leptos** — reactive WebAssembly frontend
- **SQLx + PostgreSQL** — persistence
- **Trunk** — frontend build tool

---

## API endpoints

| Method | Path | Description |
|---|---|---|
| `GET` | `/currencies` | Returns the list of available currency codes |
| `GET` | `/analyze?base=EUR&quote=USD&days=30` | Analyses the pair over the last N days (max 365) |

---

## Getting started

### Prerequisites

- Rust (stable, edition 2024)
- PostgreSQL
- [`Trunk`](https://trunkrs.dev/) for the frontend (`cargo install trunk`)
- `wasm32-unknown-unknown` target (`rustup target add wasm32-unknown-unknown`)

### Environment variables

**API / Ingestion services:**

| Variable | Required | Default | Description |
|---|---|---|---|
| `DATABASE_URL` | ✅ | — | Postgres connection string |
| `BIND_ADDR` | ❌ | `0.0.0.0:8080` | Address the API server listens on |
| `START_DATE` | ❌ | `2026-01-01T00:00:00Z` | Earliest date the ingestion service fetches rates from |
| `CURRENCIES` | ❌ | _(empty)_ | Comma-separated list of currency codes to ingest (e.g. `EUR,GBP,JPY`) |

### Start the ingestion service

```sh
cargo run -p ingestion
```

### Start the API server

```sh
cargo run -p api
```

### Start the frontend (dev)

```sh
cd crates/frontend
trunk serve
```

---

## Project conventions

- `unsafe` code is **forbidden** across the entire workspace.
- Clippy `pedantic` + `nursery` lints are enabled — the codebase is held to a high standard.
- `unwrap` / `expect` / `panic` / `todo!` are all treated as warnings, not escape hatches.
- All public items must be documented (`missing_docs = "warn"`).

---

## Running the tests

```sh
cargo test --workspace
```
