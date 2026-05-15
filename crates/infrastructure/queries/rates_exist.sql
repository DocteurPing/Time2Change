-- Returns `true` if at least one row exists for a pair within the range.
--
-- Bind order: `$1` = base, `$2` = quote, `$3` = start timestamp,
-- `$4` = end timestamp.
SELECT EXISTS (
    SELECT 1
    FROM   exchange_rates
    WHERE  base      = $1
      AND  quote     = $2
      AND  timestamp >= $3
      AND  timestamp <= $4
)
