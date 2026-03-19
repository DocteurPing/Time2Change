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

/// Saves a batch of currencies, upserting on `code`.
///
/// Bind order: `$1` = codes, `$2` = names.
pub const SAVE_CURRENCIES: &str = "
    INSERT INTO currencies (currency, name)
    SELECT UNNEST($1::TEXT[]), UNNEST($2::TEXT[])
    ON CONFLICT (currency) DO UPDATE SET name = EXCLUDED.name
";

/// Loads all currencies from the database.
pub const LOAD_CURRENCIES: &str = "
    SELECT currency, name
    FROM   currencies
    ORDER  BY currency ASC
";
