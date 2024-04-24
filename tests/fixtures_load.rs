use anyhow::Result;
use csv_async::AsyncReaderBuilder;
use tokio_stream::StreamExt;
use transactify::{
    constants::SMALL_CORRECT_FIXTURE,
    models::transaction::{Transaction, TransactionType},
    util::read_parsable_csv,
};

#[tokio::test]
async fn load_small_correct_succeeds() -> Result<()> {
    let path = format!("{}{}", env!("CARGO_MANIFEST_DIR"), SMALL_CORRECT_FIXTURE);
    let data = read_parsable_csv(path).await?;
    let mut rdr = AsyncReaderBuilder::new()
        .has_headers(true)
        .create_deserializer(data.as_bytes());
    let mut records = rdr.deserialize();
    if let Some(record) = records.next().await {
        let row: Transaction = record?;
        assert_eq!(row.transaction_type, TransactionType::Deposit);
    }
    Ok(())
}
