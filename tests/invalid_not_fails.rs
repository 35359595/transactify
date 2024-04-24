use anyhow::Result;
use transactify::{
    constants::SMALL_INVALID_SEQUENCE_FIXTURE, state::InfailableState, util::read_all_records,
};

#[tokio::test]
async fn invalid_sequence_works() -> Result<()> {
    let path = format!(
        "{}{}",
        env!("CARGO_MANIFEST_DIR"),
        SMALL_INVALID_SEQUENCE_FIXTURE
    );
    let records = read_all_records(path).await?;
    let mut state = InfailableState::new();
    state.process_transactions(records.into_iter());
    Ok(())
}
