use transactify::{
    constants::SMALL_INVALID_SEQUENCE_FIXTURE, models::balance::Balance, util::read_all_records,
};

#[tokio::test]
#[should_panic]
async fn invalid_sequence_fails() {
    let path = format!(
        "{}{}",
        env!("CARGO_MANIFEST_DIR"),
        SMALL_INVALID_SEQUENCE_FIXTURE
    );
    let records = read_all_records(path).await;
    if records.is_err() {
        return;
    }
    let mut records = records.unwrap();
    records.reverse();
    let Some(first) = records.pop() else {
        return;
    };
    let Ok(mut balance) = Balance::from_transaction(first) else {
        return;
    };
    for record in records {
        balance.process_transaction(&record).unwrap();
    }
}
