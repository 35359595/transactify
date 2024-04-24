use super::transaction::{Transaction, TransactionType};
use anyhow::{Error, Result};
use bigdecimal::{BigDecimal, Zero};
use serde::Serialize;
use std::collections::{HashMap, HashSet};

const ROUND_DIGITS: i64 = 4;

/// Balance state of each account
#[derive(Debug, Serialize, Default)]
pub struct Balance {
    /// ID of account
    pub client: u16,
    /// The total funds that are available for trading, staking, withdrawal, etc.
    /// This should be equal to the total - held amounts
    pub available: BigDecimal,
    /// The total funds that are held for dispute.
    /// This should be equal to total - available amounts
    pub held: BigDecimal,
    /// The total funds that are available or held. This should be equal to available + held
    pub total: BigDecimal,
    /// Whether the account is locked. An account is locked if a charge back occurs
    pub locked: bool,
    /// Historic view of applied transactions.
    /// WARN: Publicly invisible and should be modified by calling `self.process_transaction` or from constructor only!
    #[serde(skip)]
    historic_transactions: HashMap<u32, Transaction>,
    /// Historic view of currently disputed transactions.
    /// WARN: Publicly invisible and should be modified by calling `self.process_transaction` only!.
    #[serde(skip)]
    active_dispute_transactions: HashMap<u32, Transaction>,
    /// Transaction IDs, which were under dispute but already resolved
    #[serde(skip)]
    resolved_dispute_transactions: HashSet<u32>,
    /// Transaction IDs, which were under resolved and charged back
    #[serde(skip)]
    charged_back_transactions: HashSet<u32>,
}

impl Balance {
    /// Apply new transaction to current state
    /// Fails on:
    /// `transaction.client` is not matching `self.account_id`
    /// `self.locked` is true if transaction is not resolving the pending despute
    pub fn process_transaction(&mut self, transaction: &Transaction) -> Result<()> {
        if self.client != transaction.client {
            return Err(Error::msg("Mismatching clients for transaction"));
        }
        match transaction.transaction_type {
            TransactionType::Deposit => {
                // negative addition prevention
                if transaction.amount <= BigDecimal::zero() {
                    return Err(Error::msg("Deposit can not be negative or zero value"));
                }
                // replay prevention
                if self.historic_transactions.contains_key(&transaction.tx) {
                    return Err(Error::msg("Deposit replay attempt detected!"));
                }
                self.mint_funds(transaction.amount.to_owned());
                self.round();
                self.historic_transactions
                    .insert(transaction.tx, transaction.to_owned());
            }
            TransactionType::Withdrawal => {
                // incorrect balance check
                if self.total < transaction.amount || self.available < transaction.amount {
                    // not returning amounts on account on purpose to preserve any info leaks :)
                    return Err(Error::msg("Not enought funds to withdraw requested amount"));
                }
                // replay prevention
                if self.historic_transactions.contains_key(&transaction.tx) {
                    return Err(Error::msg("Withdraw replay attempt detected!"));
                }
                // negative substraction prevention
                if transaction.amount <= BigDecimal::zero() {
                    return Err(Error::msg("Withdrawal can not be negative or zero value"));
                }
                self.burn_funds(transaction.amount.to_owned());
                self.round();
                self.historic_transactions
                    .insert(transaction.tx, transaction.to_owned());
            }
            TransactionType::Dispute => {
                // replay prevention
                if self.resolved_dispute_transactions.contains(&transaction.tx) {
                    return Err(Error::msg("Replay Dispute not allowed"));
                }
                if let Some(historic) = self.historic_transactions.get(&transaction.tx) {
                    self.available -= historic.amount.to_owned();
                    self.held += historic.amount.to_owned();
                    self.active_dispute_transactions
                        .insert(historic.tx, historic.to_owned());
                    self.historic_transactions
                        .insert(transaction.tx, transaction.to_owned());
                }
                // assuming our partner's fault and doing nothing...
            }
            TransactionType::Resolve => {
                // sequence state preserving
                if !self.resolved_dispute_transactions.contains(&transaction.tx) {
                    return Err(Error::msg("Dispute mist be filed before Resolving it"));
                }
                let mut resolved = false;
                if let Some(disputed) = self.active_dispute_transactions.get(&transaction.tx) {
                    self.held -= disputed.amount.to_owned();
                    self.available += disputed.amount.to_owned();
                    self.resolved_dispute_transactions.insert(disputed.tx);
                    resolved = true;
                }
                if resolved {
                    self.active_dispute_transactions.remove(&transaction.tx);
                    self.historic_transactions
                        .insert(transaction.tx, transaction.to_owned());
                }
                // assuming our partner's fault and doing nothing...
            }
            TransactionType::Chargeback => {
                if !self.resolved_dispute_transactions.contains(&transaction.tx) {
                    return Err(Error::msg(
                        "To charge back dispute must be filed and resolved",
                    ));
                }
                // replay prevention
                if self.charged_back_transactions.contains(&transaction.tx) {
                    return Err(Error::msg("This transaction was already charged back."));
                }
                if let Some(historic) = self.historic_transactions.get(&transaction.tx) {
                    self.locked = true;
                    self.charged_back_transactions.insert(historic.tx);
                    self.burn_funds(historic.amount.to_owned());
                }
                // assuming our partner's fault and doing nothing...
            }
        }
        Ok(())
    }

    /// Constructor for first transaction (initial state)
    pub fn from_transaction(transaction: Transaction) -> Result<Self> {
        match transaction.transaction_type {
            TransactionType::Deposit => {
                let new_balance = transaction.amount.round(ROUND_DIGITS);
                Ok(Balance {
                    client: transaction.client,
                    available: new_balance.to_owned(),
                    total: new_balance,
                    ..Default::default()
                })
            }
            _ => Err(Error::msg(format!(
                "New balance can be instantiated from deposit only. Invalit type: {}",
                transaction.transaction_type
            ))),
        }
    }

    fn round(&mut self) {
        self.available = self.available.round(ROUND_DIGITS);
        self.total = self.total.round(ROUND_DIGITS)
    }

    fn mint_funds(&mut self, amount: BigDecimal) {
        self.available += amount.to_owned();
        self.total += amount.to_owned();
    }

    fn burn_funds(&mut self, amount: BigDecimal) {
        self.available -= amount.to_owned();
        self.total -= amount.to_owned();
    }
}

#[test]
fn decimals_are_rounded_works() {
    use std::str::FromStr;
    let mut balance = Balance::default();
    balance.mint_funds(BigDecimal::from_str("1.1111111").unwrap());
    balance.round();
    assert_eq!(balance.available, BigDecimal::from_str("1.1111").unwrap());
}
