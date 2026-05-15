-- Loads all rows for a pair within an inclusive timestamp range,
-- ordered chronologically.
--
-- Returns only the `timestamp` and `rate` columns — `base` and `quote` are
-- already known from the caller's [`CurrencyPair`] argument.
--
-- Bind order: `$1` = base, `$2` = quote, `$3` = start timestamp,
-- `$4` = end timestamp.
SELECT timestamp, rate
FROM   exchange_rates
WHERE  base      = $1
  AND  quote     = $2
  AND  timestamp >= $3
  AND  timestamp <= $4
ORDER  BY timestamp ASC
