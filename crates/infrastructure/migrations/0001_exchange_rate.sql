-- DDL to create the `exchange_rates` table if it does not already exist.
--
-- Schema:
-- ```sql
-- base       TEXT        NOT NULL  -- 3-letter ISO 4217 code, e.g. EUR
-- quote      TEXT        NOT NULL  -- 3-letter ISO 4217 code, e.g. USD
-- timestamp  TIMESTAMPTZ NOT NULL  -- moment the rate was observed
-- rate       NUMERIC     NOT NULL  -- exchange rate value, full precision
-- ```
--
-- The `(base, quote, timestamp)` triple is the natural unique key: there
-- can only be one rate per pair per point in time.
CREATE TABLE IF NOT EXISTS exchange_rates (
    base       TEXT        NOT NULL,
    quote      TEXT        NOT NULL,
    timestamp  TIMESTAMPTZ NOT NULL,
    rate       NUMERIC     NOT NULL,
    PRIMARY KEY (base, quote, timestamp)
)
