use application::ports::exchange_rate_repository::ExchangeRateRepository;
use chrono::Utc;
use domain::types::currency::Currency;
use domain::types::currency_info::CurrencyInfo;
use domain::types::currency_pair::CurrencyPair;
use domain::types::exchange_rate::ExchangeRate;
use rust_decimal::dec;
use sqlx::PgPool;

fn make_currency(code: &str) -> Currency {
    Currency::new(code).unwrap()
}

use crate::exchange_rate::repository::PostgresExchangeRateRepository;

fn make_pair() -> CurrencyPair {
    CurrencyPair::new(Currency::new("EUR").unwrap(), Currency::new("USD").unwrap()).unwrap()
}

// sqlx::test automatically spins up a temporary database,
// runs migrations, and tears it down after the test.
#[sqlx::test]
async fn save_and_load_round_trip(pool: PgPool) {
    let repo = PostgresExchangeRateRepository::new(pool);

    let now = Utc::now();
    let pair = make_pair();
    let rate = ExchangeRate::new(now, dec!(1.0850));

    repo.save_rates(&pair, &[rate]).await.unwrap();

    let range = (now - chrono::Duration::seconds(1))..=(now + chrono::Duration::seconds(1));
    let series = repo.load_rates(&pair, &range).await.unwrap();

    assert_eq!(series.rates().len(), 1);
    assert_eq!(series.rates()[0].rate(), &dec!(1.0850));

    assert!(repo.exists(&pair, &range).await.unwrap());
}

#[sqlx::test]
async fn exists_no_data(pool: PgPool) {
    let repo = PostgresExchangeRateRepository::new(pool);
    assert!(
        !repo
            .exists(&make_pair(), &(Utc::now()..=Utc::now()))
            .await
            .unwrap()
    );
}

#[sqlx::test]
async fn save_multiple_rates(pool: PgPool) {
    let repo = PostgresExchangeRateRepository::new(pool);

    let now = Utc::now();
    let pair = make_pair();
    let rates = vec![
        ExchangeRate::new(now, dec!(1.0850)),
        ExchangeRate::new(now + chrono::Duration::seconds(60), dec!(1.0860)),
        ExchangeRate::new(now + chrono::Duration::seconds(120), dec!(1.0870)),
    ];

    repo.save_rates(&pair, &rates).await.unwrap();

    let range = (now - chrono::Duration::seconds(1))..=(now + chrono::Duration::seconds(120));
    let series = repo.load_rates(&pair, &range).await.unwrap();

    assert_eq!(series.rates().len(), 3);
    assert_eq!(series.rates()[0].rate(), &dec!(1.0850));
    assert_eq!(series.rates()[1].rate(), &dec!(1.0860));
    assert_eq!(series.rates()[2].rate(), &dec!(1.0870));
}

#[sqlx::test]
async fn save_duplicate_rates(pool: PgPool) {
    let repo = PostgresExchangeRateRepository::new(pool);

    let now = Utc::now();
    let pair = make_pair();
    let rate1 = ExchangeRate::new(now, dec!(1.0850));
    let rate2 = ExchangeRate::new(now, dec!(1.0860)); // same timestamp, different rate

    repo.save_rates(&pair, std::slice::from_ref(&rate1))
        .await
        .unwrap();
    repo.save_rates(&pair, std::slice::from_ref(&rate2))
        .await
        .unwrap(); // should be ignored

    let range = (now - chrono::Duration::seconds(1))..=(now + chrono::Duration::seconds(1));
    let series = repo.load_rates(&pair, &range).await.unwrap();

    assert_eq!(series.rates().len(), 1);
    assert_eq!(series.rates()[0].rate(), &dec!(1.0850)); // original rate should be preserved
}

#[sqlx::test]
async fn save_duplicate_in_batch(pool: PgPool) {
    let repo = PostgresExchangeRateRepository::new(pool);

    let now = Utc::now();
    let pair = make_pair();
    let rate1 = ExchangeRate::new(now, dec!(1.0850));
    let rate2 = ExchangeRate::new(now, dec!(1.0860)); // same timestamp, different rate

    repo.save_rates(&pair, &[rate1.clone(), rate2.clone()])
        .await
        .unwrap(); // one of them should be ignored

    let range = (now - chrono::Duration::seconds(1))..=(now + chrono::Duration::seconds(1));
    let series = repo.load_rates(&pair, &range).await.unwrap();

    assert_eq!(series.rates().len(), 1);
    assert_eq!(series.rates()[0].rate(), &dec!(1.0850)); // original rate should be preserved
}

#[sqlx::test]
async fn save_list_currencies(pool: PgPool) {
    let repo = PostgresExchangeRateRepository::new(pool);
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
