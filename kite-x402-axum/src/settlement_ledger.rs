//! In-memory settlement ledger and reconciliation store for x402 payments.
//!
//! Records confirmed on-chain settlements, transaction hashes, amounts, and
//! payer metadata for post-settlement reconciliation and auditing.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

/// An immutable record of an x402 payment settlement event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SettlementRecord {
    /// Transaction hash of the settlement.
    pub tx_hash: String,
    /// Address of the payer.
    pub payer: String,
    /// Address that received the payment.
    pub pay_to: String,
    /// Amount in atomic token units.
    pub amount: u64,
    /// Network identifier (CAIP-2).
    pub network: String,
    /// Unix timestamp in seconds.
    pub timestamp_secs: u64,
    /// Whether the settlement succeeded.
    pub success: bool,
    /// Failure reason, if the settlement failed.
    pub error_reason: Option<String>,
}

/// Aggregate summary statistics of recorded settlements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LedgerSummary {
    /// Sum of the amounts of all successful settlements.
    pub total_settled_volume: u64,
    /// Number of successful settlements.
    pub successful_count: u64,
    /// Number of failed settlements.
    pub failed_count: u64,
}

#[derive(Debug)]
struct Inner {
    records: VecDeque<SettlementRecord>,
    by_tx: HashMap<String, SettlementRecord>,
    by_payer: HashMap<String, Vec<SettlementRecord>>,
    max_records: usize,
    total_settled_volume: u64,
    successful_count: u64,
    failed_count: u64,
}

/// Thread-safe settlement ledger.
#[derive(Debug, Clone)]
pub struct SettlementLedger {
    inner: Arc<Mutex<Inner>>,
}

impl SettlementLedger {
    /// Creates a new ledger with a maximum capacity.
    #[must_use] 
    pub fn new(max_records: usize) -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                records: VecDeque::new(),
                by_tx: HashMap::new(),
                by_payer: HashMap::new(),
                max_records: max_records.max(100),
                total_settled_volume: 0,
                successful_count: 0,
                failed_count: 0,
            })),
        }
    }

    /// Records a settlement attempt or confirmed transaction.
    pub fn record(&self, record: SettlementRecord) {
        let mut inner = self.inner.lock().expect("settlement ledger mutex poisoned");

        if record.success {
            inner.successful_count += 1;
            inner.total_settled_volume = inner.total_settled_volume.saturating_add(record.amount);
        } else {
            inner.failed_count += 1;
        }

        // Enforce capacity limit
        if inner.records.len() >= inner.max_records {
            if let Some(evicted) = inner.records.pop_front() {
                inner.by_tx.remove(&evicted.tx_hash);
                if let Some(payer_list) = inner.by_payer.get_mut(&evicted.payer) {
                    payer_list.retain(|r| r.tx_hash != evicted.tx_hash);
                }
            }
        }

        let payer_key = record.payer.to_lowercase();
        inner.by_tx.insert(record.tx_hash.clone(), record.clone());
        inner
            .by_payer
            .entry(payer_key)
            .or_default()
            .push(record.clone());
        inner.records.push_back(record);
    }

    /// Retrieves a settlement record by its transaction hash.
    #[must_use] 
    pub fn get_by_tx(&self, tx_hash: &str) -> Option<SettlementRecord> {
        let inner = self.inner.lock().expect("settlement ledger mutex poisoned");
        inner.by_tx.get(tx_hash).cloned()
    }

    /// Returns all records for a given payer address.
    #[must_use] 
    pub fn find_by_payer(&self, payer: &str) -> Vec<SettlementRecord> {
        let inner = self.inner.lock().expect("settlement ledger mutex poisoned");
        inner
            .by_payer
            .get(&payer.to_lowercase())
            .cloned()
            .unwrap_or_default()
    }

    /// Returns aggregate summary metrics.
    #[must_use] 
    pub fn summary(&self) -> LedgerSummary {
        let inner = self.inner.lock().expect("settlement ledger mutex poisoned");
        LedgerSummary {
            total_settled_volume: inner.total_settled_volume,
            successful_count: inner.successful_count,
            failed_count: inner.failed_count,
        }
    }

    /// Number of entries currently stored in the ledger.
    #[must_use] 
    pub fn len(&self) -> usize {
        let inner = self.inner.lock().expect("settlement ledger mutex poisoned");
        inner.records.len()
    }

    /// Whether the ledger is empty.
    #[must_use] 
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Default for SettlementLedger {
    fn default() -> Self {
        Self::new(5000)
    }
}
