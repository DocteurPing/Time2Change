#[test]
fn mock_var(vars: &[(&str, &str)]) -> impl Fn(&str) -> Result<String, env::VarError> {
    |key: &str| {
        vars.iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v.to_string())
            .ok_or(env::VarError::NotPresent)
    }
}

#[test]
fn test_from_env_success() {
    let vars = vec![
        ("DATABASE_URL", "postgres://localhost"),
        ("CURRENCIES", "EUR,GBP"),
    ];
    let config = IngestionConfig::from_env_impl(mock_var(&vars)).unwrap();

    assert_eq!(config.database_url(), "postgres://localhost");
    assert_eq!(
        *config.start_date(),
        "2026-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap()
    );
    assert_eq!(
        config.list_currencies(),
        [Currency::new("EUR").unwrap(), Currency::new("GBP").unwrap()]
            .into_iter()
            .collect::<HashSet<_>>()
    );
    assert_eq!(
        config.backfill_interval(),
        Duration::from_secs(DEFAULT_BACKFILL_INTERVAL_SECS)
    );
    assert_eq!(
        config.poll_interval(),
        Duration::from_secs(DEFAULT_POLL_INTERVAL_SECS)
    );
}

#[test]
fn test_from_env_with_custom_start_date() {
    let vars = vec![
        ("DATABASE_URL", "postgres://localhost"),
        ("START_DATE", "2024-06-15T12:30:00Z"),
    ];
    let config = IngestionConfig::from_env_impl(mock_var(&vars)).unwrap();

    assert_eq!(
        *config.start_date(),
        "2024-06-15T12:30:00Z".parse::<DateTime<Utc>>().unwrap()
    );
    assert_eq!(config.list_currencies, HashSet::new());
}

#[test]
fn test_from_env_missing_database_url() {
    let vars: Vec<(&str, &str)> = vec![];
    let result = IngestionConfig::from_env_impl(mock_var(&vars));

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        "DATABASE_URL environment variable is required"
    );
}

#[test]
fn test_from_env_invalid_start_date() {
    let vars = vec![
        ("DATABASE_URL", "postgres://localhost"),
        ("START_DATE", "not-a-date"),
    ];
    let result = IngestionConfig::from_env_impl(mock_var(&vars));

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("invalid START_DATE"));
}

#[test]
fn test_from_env_default() {
    let result = IngestionConfig::from_env().unwrap();
    assert_eq!(
        *result.start_date(),
        "2026-01-01T00:00:00Z".parse::<DateTime<Utc>>().unwrap()
    );
    assert_eq!(
        result.backfill_interval(),
        Duration::from_secs(DEFAULT_BACKFILL_INTERVAL_SECS)
    );
    assert_eq!(
        result.poll_interval(),
        Duration::from_secs(DEFAULT_POLL_INTERVAL_SECS)
    );
}

#[test]
fn test_from_env_custom_intervals() {
    let vars = vec![
        ("DATABASE_URL", "postgres://localhost"),
        ("BACKFILL_INTERVAL_SECS", "5"),
        ("POLL_INTERVAL_SECS", " 905 "),
    ];
    let config = IngestionConfig::from_env_impl(mock_var(&vars)).unwrap();

    assert_eq!(config.backfill_interval(), Duration::from_secs(5));
    assert_eq!(config.poll_interval(), Duration::from_secs(905));
}

#[test]
fn test_from_env_rejects_zero_interval() {
    let vars = vec![
        ("DATABASE_URL", "postgres://localhost"),
        ("POLL_INTERVAL_SECS", "0"),
    ];
    let result = IngestionConfig::from_env_impl(mock_var(&vars));

    assert_eq!(
        result.unwrap_err(),
        "invalid POLL_INTERVAL_SECS: must be greater than zero"
    );
}

#[test]
fn test_from_env_rejects_non_numeric_interval() {
    let vars = vec![
        ("DATABASE_URL", "postgres://localhost"),
        ("BACKFILL_INTERVAL_SECS", "soon"),
    ];
    let result = IngestionConfig::from_env_impl(mock_var(&vars));

    assert!(
        result
            .unwrap_err()
            .contains("invalid BACKFILL_INTERVAL_SECS")
    );
}
