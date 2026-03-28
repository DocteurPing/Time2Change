//! Core domain value objects and aggregates used throughout the application.
//!
//! This module groups the strongly typed building blocks that represent
//! currencies, currency pairs, exchange rates, time series data, and quality
//! assessments for imported rates.

/// Currency domain type and its validation errors.
pub mod currency;

/// Currency pair domain type and its validation errors.
pub mod currency_pair;

/// Exchange rate value object with timestamped rate data.
pub mod exchange_rate;

/// Rate quality result types, including overall score and score breakdown.
pub mod rate_quality;

/// Configuration types for rate quality calculation.
pub mod rate_quality_config;

/// Time-series aggregate for exchange rates and quality analysis.
pub mod time_series;

/// Currency information (symbol and name).
pub mod currency_info;

/// Utility functions for working with currency pairs.
pub mod utils;
