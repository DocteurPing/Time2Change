/// DDL to create the `exchange_rates` table if it does not already exist.
///
/// Schema:
/// ```sql
/// base       TEXT        NOT NULL  -- 3-letter ISO 4217 code, e.g. EUR
/// quote      TEXT        NOT NULL  -- 3-letter ISO 4217 code, e.g. USD
/// timestamp  TIMESTAMPTZ NOT NULL  -- moment the rate was observed
/// rate       NUMERIC     NOT NULL  -- exchange rate value, full precision
/// ```
///
/// The `(base, quote, timestamp)` triple is the natural unique key: there
/// can only be one rate per pair per point in time.
pub const CREATE_TABLE: &str = "
    CREATE TABLE IF NOT EXISTS exchange_rates (
        base       TEXT        NOT NULL,
        quote      TEXT        NOT NULL,
        timestamp  TIMESTAMPTZ NOT NULL,
        rate       NUMERIC     NOT NULL,
        PRIMARY KEY (base, quote, timestamp)
    )
";

/// Inserts a batch of exchange rate rows.
///
/// Uses `ON CONFLICT DO NOTHING` so that re-ingesting the same
/// `(base, quote, timestamp)` triple is a silent no-op rather than an error.
/// Change to `ON CONFLICT ... DO UPDATE` for upsert semantics.
///
/// Bind order: `$1` = base, `$2` = quote, `$3` = timestamp, `$4` = rate.
pub const INSERT_RATE: &str = "
    INSERT INTO exchange_rates (base, quote, timestamp, rate)
    SELECT $1, $2, UNNEST($3::TIMESTAMPTZ[]), UNNEST($4::NUMERIC[])
    ON CONFLICT (base, quote, timestamp) DO NOTHING
";

/// Loads all rows for a pair within an inclusive timestamp range,
/// ordered chronologically.
///
/// Returns only the `timestamp` and `rate` columns — `base` and `quote` are
/// already known from the caller's [`CurrencyPair`] argument.
///
/// Bind order: `$1` = base, `$2` = quote, `$3` = start timestamp,
/// `$4` = end timestamp.
pub const LOAD_RATES: &str = "
    SELECT timestamp, rate
    FROM   exchange_rates
    WHERE  base      = $1
      AND  quote     = $2
      AND  timestamp >= $3
      AND  timestamp <= $4
    ORDER  BY timestamp ASC
";

/// Returns `true` if at least one row exists for a pair within the range.
///
/// Bind order: `$1` = base, `$2` = quote, `$3` = start timestamp,
/// `$4` = end timestamp.
pub const EXISTS: &str = "
    SELECT EXISTS (
        SELECT 1
        FROM   exchange_rates
        WHERE  base      = $1
          AND  quote     = $2
          AND  timestamp >= $3
          AND  timestamp <= $4
    )
";
