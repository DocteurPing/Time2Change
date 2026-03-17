//! This module contains the implementation of the rate provider repository

/// Data transfer objects used by the rate provider repository for representing exchange rate requests and responses.
pub mod dto;

/// Concrete implementation of the rate provider repository, which fetches exchange rates from the Frankfurter API.
pub mod frankfurter;
