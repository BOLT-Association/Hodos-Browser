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
    let Some(provider) = ChaintracksProvider::from_env(http_client(crate::services::CallClass::IndexerAsync.timeout())) else {
        return Ok(());
    };
    let network = svc.chain.lock().unwrap().params().name;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as u32)
        .unwrap_or(0);

    let mut fresh: Vec<Entry> = Vec::new();
    let (report, result) =
        header_sync::sync_once_reporting(&svc.chain, &ChaintracksSource(provider), now, &mut |e| fresh.push(e.clone())).await;

    // Act on reorgs even when the run then failed: the chain has already switched branches in
    // memory, and a later run will not see a reorg to report.
    for r in &report.reorgs {
        if let AddOutcome::Reorg { fork_height, depth, old_tip, new_tip } = r {
            header_sync::request_proof_recheck(fork_height + 1);
            warn!(
                "🔀 Header chain REORG: depth {} from height {} ({} -> {}). Re-checking stored proofs above the fork.",
                depth, fork_height, old_tip, new_tip
            );
        }
    }
    if !fresh.is_empty() {
        let db = state.database.lock().map_err(|e| format!("DB lock: {}", e))?;
        let repo = HeaderChainRepository::new(db.connection());
        let rows: Vec<(String, u32, String)> =
            fresh.iter().map(|e| (e.hash.clone(), e.height, e.header.to_hex())).collect();
        repo.insert_many(network, &rows)
            .map_err(|err| format!("persist {} header(s): {}", rows.len(), err))?;
    }

    result?;
    // The chain may now be able to judge proofs that were held waiting for these headers.
    super::task_check_for_proofs::resolve_held(state);
    header_sync::request_startup_recheck_once();
    if report.added > 0 {
        info!("🔗 Header chain: +{} header(s), tip {:?}", report.added, report.tip_height);
    }
    Ok(())
}

/// The sync runs inside the Monitor tick while holding `RUN_LOCK`, so a request that never
/// answers would stall every other task and the push follow-ups. Always bound it.
fn http_client(timeout: std::time::Duration) -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(timeout)
        .build()
        .unwrap_or_else(|_| reqwest::Client::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncReadExt;

    #[tokio::test]
    async fn a_chaintracks_server_that_never_answers_fails_the_request_instead_of_hanging() {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut sock, _) = listener.accept().await.unwrap();
            let mut buf = [0u8; 1024];
            let _ = sock.read(&mut buf).await; // accept, read, then say nothing
            tokio::time::sleep(std::time::Duration::from_secs(30)).await;
        });
        let client = http_client(std::time::Duration::from_millis(150));
        let started = std::time::Instant::now();
        let r = client.get(format!("http://{}/height", addr)).send().await;
        assert!(r.is_err(), "must time out");
        assert!(started.elapsed() < std::time::Duration::from_secs(5));
    }
}
