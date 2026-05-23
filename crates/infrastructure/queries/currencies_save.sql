-- Saves a batch of currencies, replacing rows not in this batch.
-- Bind order: `$1` = currency codes, `$2` = currency names.
WITH incoming AS (
    SELECT UNNEST($1::TEXT[]) AS currency, UNNEST($2::TEXT[]) AS name
),
deleted AS (
    DELETE FROM currencies
    WHERE currency NOT IN (SELECT currency FROM incoming)
)
INSERT INTO currencies (currency, name)
SELECT currency, name
FROM incoming
ON CONFLICT (currency) DO UPDATE
SET name = EXCLUDED.name;
