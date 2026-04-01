use application::ports::currency_repository::CurrencyRepository;
use domain::types::currency::Currency;
use domain::types::currency_info::CurrencyInfo;
use sqlx::PgPool;

use crate::currency::repository::PostgresCurrencyRepository;
use crate::exchange_rate::repository::PostgresExchangeRateRepository;

fn make_currency(code: &str) -> Currency {
    Currency::new(code).unwrap()
}

// sqlx::test automatically spins up a temporary database,
// runs migrations, and tears it down after the test.
#[sqlx::test]
async fn save_and_list_currencies_round_trip(pool: PgPool) {
    let migrator = PostgresExchangeRateRepository::new(pool.clone());
    migrator.migrate().await.unwrap();

    let repo = PostgresCurrencyRepository::new(pool);
    let currencies = vec![
        CurrencyInfo::new(make_currency("USD"), "US Dollar".to_owned()),
        CurrencyInfo::new(make_currency("EUR"), "Euro".to_owned()),
    ];

    repo.save_currencies(&currencies).await.unwrap();

    let saved = repo.list_currencies().await.unwrap();
    assert_eq!(saved.len(), 2);
    assert_eq!(saved[0].code().to_string(), "EUR");
    assert_eq!(saved[0].name(), "Euro");
    assert_eq!(saved[1].code().to_string(), "USD");
    assert_eq!(saved[1].name(), "US Dollar");
}

#[sqlx::test]
async fn save_currencies_empty_input_is_noop(pool: PgPool) {
    let migrator = PostgresExchangeRateRepository::new(pool.clone());
    migrator.migrate().await.unwrap();

    let repo = PostgresCurrencyRepository::new(pool);

    repo.save_currencies(&[]).await.unwrap();

    let saved = repo.list_currencies().await.unwrap();
    assert!(saved.is_empty());
}

#[sqlx::test]
async fn save_currencies_upsert_updates_existing_name(pool: PgPool) {
    let migrator = PostgresExchangeRateRepository::new(pool.clone());
    migrator.migrate().await.unwrap();

    let repo = PostgresCurrencyRepository::new(pool);

    let first = vec![CurrencyInfo::new(
        make_currency("EUR"),
        "Euro (old)".to_owned(),
    )];
    repo.save_currencies(&first).await.unwrap();

    let second = vec![CurrencyInfo::new(make_currency("EUR"), "Euro".to_owned())];
    repo.save_currencies(&second).await.unwrap();

    let saved = repo.list_currencies().await.unwrap();
    assert_eq!(saved.len(), 1);
    assert_eq!(saved[0].code().to_string(), "EUR");
    assert_eq!(saved[0].name(), "Euro");
}
