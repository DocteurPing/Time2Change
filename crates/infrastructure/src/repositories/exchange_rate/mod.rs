/// Repository implementation for exchange rate data, including database models, and related components.
pub mod model;

/// Database queries and operations related to exchange rate data.
pub mod queries;

/// Concrete repository implementation for exchange rate data made with `PostgreSQL` and `SQLx`.
pub mod repository;

/// Function to convert Database error to application-level `RepositoryError`.
pub mod error;
