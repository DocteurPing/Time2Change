-- DDL to create the `currencies` table if it does not already exist.
--
-- Schema:
-- ```sql
-- currency   TEXT        NOT NULL  -- 3-letter ISO 4217 code, e.g. EUR
-- name       TEXT        NOT NULL  -- Full name of the currency
-- ```
CREATE TABLE IF NOT EXISTS currencies (
    currency    TEXT        NOT NULL,
    name        TEXT        NOT NULL,
    PRIMARY KEY (currency)
)
