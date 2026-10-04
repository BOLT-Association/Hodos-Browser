//! Push driver (spv mode): owns the Arcade SSE client and, when it wakes us, runs the header
//! sync and proof check immediately, retrying a few times quickly because the header for the
//! just-mined block can trail Arcade's MINED event by a couple of seconds.
//!
//! This never replaces polling. `TaskCheckForProofs` still runs on the monitor's schedule, and
//! it is the only code that verifies and stores a proof; push just makes it run sooner.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use actix_web::web;
use log::{info, warn};

use crate::arcade_push;
use crate::AppState;

static STARTED: AtomicBool = AtomicBool::new(false);

/// Quick retries after a wake-up while the proof task is still waiting for the header chain.
const FOLLOW_UP_ATTEMPTS: u32 = 8;
const FOLLOW_UP_GAP: Duration = Duration::from_secs(2);

/// Start the driver once. No-op unless push is enabled (spv mode, SSE URL, not switched off).
pub fn start(state: web::Data<AppState>) {
    if !crate::chain_mode::push_enabled() {
        if crate::chain_mode::is_spv() {
            info!("📡 Arcade push: off (no HODOS_ARCADE_SSE_URL or HODOS_ARCADE_PUSH=off) — polling only");
        }
        return;
    }
    if STARTED.swap(true, Ordering::SeqCst) {
        return;
    }
    let Some(sse_url) = crate::chain_mode::arcade_sse_url() else { return };
    info!("📡 Arcade push: on ({}) — polling stays as the safety net", sse_url);

    // The SSE client: it waits for a token, so it can be started before the wallet is unlocked.
    {
        let shutdown = state.shutdown.clone();
        tokio::spawn(arcade_push::run_client(
            sse_url,
            arcade_push::token,
            |st| {
                info!("📡 Arcade push: {} {} — waking the proof check", &st.txid[..st.txid.len().min(16)], st.tx_status);
                arcade_push::buffer_event(st); // keep the MINED payload (merkle path) for the driver
                arcade_push::nudge();
            },
            shutdown,
            arcade_push::ClientOptions::default(),
        ));
    }

    // Token provider + wake-up handler.
    tokio::spawn(async move {
        let http = reqwest::Client::builder()
            .timeout(crate::services::CallClass::IndexerAsync.timeout())
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        let mut token_ticker = tokio::time::interval(Duration::from_secs(5));
        loop {
            tokio::select! {
                _ = state.shutdown.cancelled() => return,
                _ = token_ticker.tick() => ensure_token(&state),
                _ = arcade_push::nudged() => {
                    hold_pushed_proofs(&state);
                    follow_up(&state, &http).await
                }
            }
        }
    });
}

/// Put the proofs carried by MINED events into `pending_proofs`; the header sync that follows verifies
/// them locally (no re-fetch). An event that carries no proof just wakes the normal proof check.
fn hold_pushed_proofs(state: &web::Data<AppState>) {
    let events = arcade_push::take_events();
    if events.is_empty() {
        return;
    }
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    if let Ok(db) = state.database.lock() {
        let held = crate::pending_proofs::hold_events(db.connection(), &events, now);
        if held > 0 {
            info!("📡 Arcade push: holding {} proof(s) until the header chain can verify them", held);
        }
    }
}

/// Derive the callback token once the wallet's master key is readable (after create / unlock).
fn ensure_token(state: &web::Data<AppState>) {
    if arcade_push::token().is_some() {
        return;
    }
    let key = match state.database.try_lock() {
        Ok(db) => crate::database::get_master_private_key_from_db(&db).ok(),
        Err(_) => None, // busy; try again in 5 s
    };
    if let Some(key) = key {
        arcade_push::set_token(arcade_push::derive_token(&key));
        info!("📡 Arcade push: callback token ready");
    }
}

/// Header sync, then proof check; repeat quickly while a mined tx is still waiting for its header.
async fn follow_up(state: &web::Data<AppState>, http: &reqwest::Client) {
    for attempt in 0..FOLLOW_UP_ATTEMPTS {
        if let Err(e) = super::task_sync_headers::run(state).await {
            warn!("   ⚠️ push follow-up: header sync failed: {}", e);
        }
        if let Err(e) = super::task_check_for_proofs::run(state, http).await {
            warn!("   ⚠️ push follow-up: proof check failed: {}", e);
        }
        if !arcade_push::proof_waiting() {
            return;
        }
        if attempt + 1 < FOLLOW_UP_ATTEMPTS {
            tokio::time::sleep(FOLLOW_UP_GAP).await;
        }
    }
    // Still waiting: the regular poll will pick it up.
}
