/// Saves a batch of currencies, upserting on `currency`.
///
/// Bind order: `$1` = currency codes, `$2` = currency names.
pub const SAVE_CURRENCIES: &str = "
    INSERT INTO currencies (currency, name)
    SELECT UNNEST($1::TEXT[]), UNNEST($2::TEXT[])
    ON CONFLICT (currency) DO UPDATE
    SET name = EXCLUDED.name
";

/// Loads all persisted currencies sorted by code.
pub const LOAD_CURRENCIES: &str = "
    SELECT currency, name
    FROM   currencies
    ORDER  BY currency ASC
";
