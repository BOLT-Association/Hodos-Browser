//! Tip stream driver (spv mode): owns the chaintracks tip stream client and, when the tip changes,
//! runs the header sync and then the proof check at once instead of at the next monitor tick.
//!
//! This never replaces the schedule. `TaskSyncHeaders` and `TaskCheckForProofs` still run on the
//! monitor's intervals, and they are the only code that adds a header or stores a proof; the
//! stream just makes them run sooner. Both take their own run lock, so a stream-triggered run and
//! the monitor's tick never overlap.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use actix_web::web;
use log::{info, warn};
use tokio::sync::Notify;

use crate::tip_stream;
use crate::AppState;

static STARTED: AtomicBool = AtomicBool::new(false);
static NEW_TIP: Notify = Notify::const_new();

/// Start the driver once. No-op unless the tip stream is enabled (spv mode, not switched off).
pub fn start(state: web::Data<AppState>) {
    if !crate::chain_mode::tip_stream_enabled() {
        if crate::chain_mode::is_spv() {
            info!("🔗 Tip stream: off (HODOS_TIP_STREAM=off) — header sync on the schedule only");
        }
        return;
    }
    if STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    let Some(base) = crate::services::providers::chaintracks::configured_base_url() else { return };
    let url = tip_stream::stream_url(&base);
    info!("🔗 Tip stream: on ({}) — the scheduled header sync stays as the safety net", url);

    {
        // The frame is compared with the last one and otherwise unused (see `tip_stream`).
        let last: Mutex<Option<String>> = Mutex::new(None);
        tokio::spawn(tip_stream::run_client(
            url,
            move |frame| {
                let mut last = last.lock().unwrap_or_else(|e| e.into_inner());
                if tip_stream::is_new_tip(&mut last, frame) {
                    NEW_TIP.notify_one();
                }
            },
            state.shutdown.clone(),
            tip_stream::ClientOptions::default(),
        ));
    }

    tokio::spawn(async move {
        let http = reqwest::Client::builder()
            .timeout(crate::services::CallClass::IndexerAsync.timeout())
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        loop {
            tokio::select! {
                _ = state.shutdown.cancelled() => return,
                _ = NEW_TIP.notified() => {
                    if let Err(e) = super::task_sync_headers::run(&state).await {
                        warn!("   ⚠️ tip stream: header sync failed: {}", e);
                    }
                    if let Err(e) = super::task_check_for_proofs::run(&state, &http).await {
                        warn!("   ⚠️ tip stream: proof check failed: {}", e);
                    }
                }
            }
        }
    });
}
