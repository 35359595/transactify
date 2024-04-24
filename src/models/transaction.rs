//! Transaction releated code

use bigdecimal::BigDecimal;
use serde::Deserialize;
use std::{fmt::Display, str::FromStr};

/// Transaction representation from DB(csv)
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub struct Transaction {
    pub client: u16,
    pub tx: u32,
    #[serde(deserialize_with = "default_if_empty")]
    pub amount: BigDecimal,
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
}

fn default_if_empty<'de, D, T>(de: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de> + Default,
{
    Option::<T>::deserialize(de).map(|x| x.unwrap_or_else(|| T::default()))
}

/// Type of transaction dictates how balances or other info in Transaction is applied to account's balances
#[derive(Debug, Deserialize, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    /// A deposit is a credit to the client's asset account,
    /// meaning it should increase the available and total funds of the client account
    Deposit,
    /// A withdraw is a debit to the client's asset account,
    /// meaning it should decrease the available and total funds of the client account
    Withdrawal,
    ///A dispute represents a client's claim that a transaction was erroneous and should be reversed.
    /// The transaction shouldn't be reversed yet but the associated funds should be held.
    /// This means that the clients available funds should decrease by the amount disputed,
    /// their held funds should increase by the amount disputed,
    /// while their total funds should remain the same.
    Dispute,
    /// A resolve represents a resolution to a dispute, releasing the associated held funds.
    /// Funds that were previously disputed are no longer disputed.
    /// This means that the clients held funds should decrease by the amount no longer disputed,
    /// their available funds should increase by the amount no longer disputed,
    /// and their total funds should remain the same.
    Resolve,
    /// A chargeback is the final state of a dispute and represents the client reversing a transaction.
    /// Funds that were held have now been withdrawn.
    /// This means that the clients held funds and total funds should decrease by the amount previously disputed.
    /// If a chargeback occurs the client's account should be immediately frozen.
    Chargeback,
}

impl Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Deposit => f.write_str("deposit"),
            Self::Withdrawal => f.write_str("withdrawal"),
            Self::Chargeback => f.write_str("chargeback"),
            Self::Dispute => f.write_str("dispute"),
            Self::Resolve => f.write_str("resolve"),
        }
    }
}
impl FromStr for TransactionType {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "deposit" => Ok(Self::Deposit),
            "withdrawal" => Ok(Self::Withdrawal),
            "dispute" => Ok(Self::Dispute),
            "resolve" => Ok(Self::Resolve),
            "chargeback" => Ok(Self::Chargeback),
            _ => Err(anyhow::Error::msg(format!(
                "unknown transaction type {}",
                s
            ))),
        }
    }
}
