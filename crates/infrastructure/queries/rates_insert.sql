-- Inserts a batch of exchange rate rows.
--
-- Uses `ON CONFLICT DO NOTHING` so that re-ingesting the same
-- `(base, quote, timestamp)` triple is a silent no-op rather than an error.
-- Change to `ON CONFLICT ... DO UPDATE` for upsert semantics.
--
-- Bind order: `$1` = base, `$2` = quote, `$3` = timestamp, `$4` = rate.
INSERT INTO exchange_rates (base, quote, timestamp, rate)
SELECT $1, $2, UNNEST($3::TIMESTAMPTZ[]), UNNEST($4::NUMERIC[])
ON CONFLICT (base, quote, timestamp) DO NOTHING
