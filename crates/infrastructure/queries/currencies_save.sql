-- Saves a batch of currencies, upserting on `currency`.
-- Bind order: `$1` = currency codes, `$2` = currency names.
INSERT INTO currencies (currency, name)
SELECT UNNEST($1::TEXT[]), UNNEST($2::TEXT[])
ON CONFLICT (currency) DO UPDATE
SET name = EXCLUDED.name;
