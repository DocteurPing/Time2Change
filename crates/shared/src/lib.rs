//! Shared utilities and cross-cutting abstractions for the `Time2Change` workspace.
//!
//! This crate is intended to hold functionality that is reused across multiple
//! workspace crates but does not belong to a single domain, application, or
//! infrastructure boundary.
//!
//! Typical examples of code that may live here include:
//! - common error and result types,
//! - serialization helpers,
//! - workspace-wide constants,
//! - tracing and logging helpers,
//! - small utility traits or functions used in several crates.
//!
//! The crate is currently a placeholder and does not yet expose any public API.

pub mod config;
