use crate::{
    models::{balance::Balance, transaction::Transaction},
    util::write_all_records,
};
use anyhow::Result;
use log::error;
use std::{collections::HashMap, path::Path};

const STATE_ERROR: &str = "State";

#[derive(Debug, Default)]
pub struct InfailableState {
    // WARN: is private to avoid mishaps. Should be mutated by `self.process_transactions()` only!
    accounts: HashMap<u16, Balance>,
}

impl InfailableState {
    /// Generic constructor based on `Default::default()`
    pub fn new() -> Self {
        Default::default()
    }

    /// High level abstraction interface for processing all the collected transactions without termination on failed ones
    pub fn process_transactions(&mut self, transactions: impl Iterator<Item = Transaction>) {
        transactions.into_iter().map(|t|{
            let client_id = t.client;
            if let Some(state) = self.accounts.get_mut(&client_id) {
                if let Err(e) = state.process_transaction(&t) {
                    error!(target: STATE_ERROR, "Failed to apply transaction to existing state with reason: {}", e);
                }
            }
            else {
                match Balance::from_transaction(t) {
                    Ok(b) => drop(self.accounts.insert(client_id, b)),
                    Err(e) =>
                        error!(target: STATE_ERROR, "Failed to build new state from transaction with reason: {}", e)
                }
            }
        }).for_each(drop);
    }

    /// Saves state as balances into given file location
    pub async fn store_to_file(&self, path: impl AsRef<Path>) -> Result<()> {
        write_all_records(path, self.accounts.values()).await
    }
}
