//! Placeholder for the raw-tx, outspend and UTXO chains in spv mode.
//!
//! Arcade has no raw-tx, outspend or address-UTXO endpoint, and spv mode forbids public
//! indexers. Every operation therefore returns an error (never `NotFound`, which the
//! chain treats as a positive "does not exist" signal). Raw transactions and outputs
//! reach the wallet inside BEEFs instead.

use async_trait::async_trait;

use crate::services::provider::{IndexerError, IndexerProvider, ProviderOp};

const NAME: &str = "spv_no_indexer";

pub struct SpvNoIndexerProvider;

fn unavailable(what: &str) -> IndexerError {
    IndexerError::InvalidResponse {
        provider: NAME,
        reason: format!("{} is unavailable in spv mode (no public indexer; data arrives in BEEFs)", what),
    }
}

#[async_trait]
impl IndexerProvider for SpvNoIndexerProvider {
    fn name(&self) -> &'static str {
        NAME
    }

    fn supports(&self, _op: ProviderOp) -> bool {
        true // claim everything so the caller gets this explicit error, not a silent skip
    }

    async fn get_raw_tx(&self, _txid: &str) -> Result<Vec<u8>, IndexerError> {
        Err(unavailable("raw transaction lookup"))
    }

    async fn outspend(&self, _txid: &str, _vout: u32) -> Result<crate::services::provider::OutspendStatus, IndexerError> {
        Err(unavailable("outspend lookup"))
    }

    async fn fetch_utxos(&self, _address: &str) -> Result<Vec<crate::utxo_fetcher::UTXO>, IndexerError> {
        Err(unavailable("address UTXO lookup"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn every_answer_is_an_error_and_never_not_found() {
        let p = SpvNoIndexerProvider;
        assert!(!matches!(p.get_raw_tx("ab").await, Err(IndexerError::NotFound) | Ok(_)));
        assert!(!matches!(p.outspend("ab", 0).await, Err(IndexerError::NotFound) | Ok(_)));
        assert!(!matches!(p.fetch_utxos("addr").await, Err(IndexerError::NotFound) | Ok(_)));
    }
}
