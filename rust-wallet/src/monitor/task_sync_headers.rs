//! TaskSyncHeaders — keep the verified header chain (WS4) in step with the
//! configured chaintracks server.
//!
//! No-op unless `HODOS_CHAINTRACKS_URL` is set and `header_sync::init` ran at
//! startup. The server only says what exists; `HeaderChain` validates every header
//! and picks the heaviest branch itself. New headers are persisted to the V26
//! `header_chain` table even when the sync stops part-way on an invalid one, so the
//! good prefix is kept.

use actix_web::web;
use log::{info, warn};

use crate::database::HeaderChainRepository;
use crate::header_chain::{AddOutcome, Entry};
use crate::header_sync::{self, ChaintracksSource};
use crate::services::providers::ChaintracksProvider;
use crate::AppState;

/// One header sync at a time: the monitor's tick and the push driver both call `run`.
static RUN_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

pub async fn run(state: &web::Data<AppState>) -> Result<(), String> {
    let _one_at_a_time = RUN_LOCK.lock().await;
    let Some(svc) = header_sync::global() else {
        return Ok(());
    };
    let Some(provider) = ChaintracksProvider::from_env(reqwest::Client::new()) else {
        return Ok(());
    };
    let network = svc.chain.lock().unwrap().params().name;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as u32)
        .unwrap_or(0);

    let mut fresh: Vec<Entry> = Vec::new();
    let result =
        header_sync::sync_once(&svc.chain, &ChaintracksSource(provider), now, &mut |e| fresh.push(e.clone())).await;

    if !fresh.is_empty() {
        let db = state.database.lock().map_err(|e| format!("DB lock: {}", e))?;
        let repo = HeaderChainRepository::new(db.connection());
        for e in &fresh {
            repo.insert(network, &e.hash, e.height, &e.header.to_hex())
                .map_err(|err| format!("persist header {}: {}", e.height, err))?;
        }
    }

    let report = result?;
    header_sync::request_startup_recheck_once();
    for r in &report.reorgs {
        if let AddOutcome::Reorg { fork_height, depth, old_tip, new_tip } = r {
            header_sync::request_proof_recheck(fork_height + 1);
            warn!(
                "🔀 Header chain REORG: depth {} from height {} ({} -> {}). Re-checking stored proofs above the fork.",
                depth, fork_height, old_tip, new_tip
            );
        }
    }
    if report.added > 0 {
        info!("🔗 Header chain: +{} header(s), tip {:?}", report.added, report.tip_height);
    }
    Ok(())
}
