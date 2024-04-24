use crate::models::{balance::Balance, transaction::Transaction};
use anyhow::Result;
use csv_async::{AsyncReaderBuilder, AsyncSerializer};
use log::info;
use std::path::Path;
use tokio::{fs::File, io::AsyncReadExt};
use tokio_stream::StreamExt;

/// Reads content of given file by path and parses it into csv de-serializable `String`
pub async fn read_parsable_csv(path: impl AsRef<Path>) -> Result<String> {
    let mut file = File::open(&path).await?;
    let mut data = String::default();
    file.read_to_string(&mut data).await?;
    Ok(data.replace(' ', ""))
}

/// Reads all records from given file by path
pub async fn read_all_records(path: impl AsRef<Path>) -> Result<Vec<Transaction>> {
    let data = read_parsable_csv(path).await?;
    let mut rdr = AsyncReaderBuilder::new()
        .has_headers(true)
        .create_deserializer(data.as_bytes());
    let mut records = rdr.deserialize();
    let mut all = vec![];
    while let Some(record) = records.next().await {
        let row: Transaction = record?;
        all.push(row);
    }
    Ok(all)
}

pub async fn write_all_records(
    path: impl AsRef<Path>,
    records: impl Iterator<Item = &Balance>,
) -> Result<()> {
    let file = File::create(path).await?;
    let mut serializer = AsyncSerializer::from_writer(file);
    let iter = records.into_iter();
    for record in iter {
        info!(
            "Storing state record for client {} into file",
            record.client
        );
        serializer.serialize(record).await?;
    }
    Ok(())
}
