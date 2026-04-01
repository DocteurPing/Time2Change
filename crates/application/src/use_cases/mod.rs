//! Application use cases that orchestrate domain logic through ports.
//!
//! This module exposes the primary workflows supported by the application
//! layer, including historical pair analysis and exchange-rate ingestion.

/// Use case for analyzing a currency pair over a historical lookback window.
pub mod analyze_pair;

/// Use case for fetching and persisting the latest exchange rate for a pair.
pub mod ingest_rates;

/// Use case for fetching and persisting the list of available currencies.
pub mod sync_currencies;
