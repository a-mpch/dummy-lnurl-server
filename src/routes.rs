use crate::State;
use axum::extract::{Path, Query};
use axum::http::StatusCode;
use axum::{Extension, Json};
use bitcoin::hashes::{sha256, Hash};
use lnurl::pay::PayResponse;
use lnurl::Tag;
use rand::Rng;
use serde_json::{json, Value};
use std::collections::HashMap;

/// Generates a dummy BOLT11 invoice string.
/// This invoice looks valid but will fail when attempting to pay.
fn generate_dummy_invoice(amount_msats: u64, hash: &str) -> String {
    let amount_sats = amount_msats / 1000;
    let random_suffix: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(50)
        .map(char::from)
        .collect();

    // Generate a fake BOLT11-style string
    // Real invoices are bech32-encoded, but this dummy just needs to be recognizable
    format!(
        "lnbc{}n1p{}{}",
        amount_sats,
        &hash[..8],
        random_suffix.to_lowercase()
    )
}

/// HTTP endpoint for generating dummy Lightning invoices.
pub async fn get_invoice(
    Path(hash): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    Extension(state): Extension<State>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let amount_msats = match params.get("amount").and_then(|a| a.parse::<u64>().ok()) {
        None => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "status": "ERROR",
                    "reason": "Missing amount parameter",
                })),
            ))
        }
        Some(amount) => amount,
    };

    let dummy_invoice = generate_dummy_invoice(amount_msats, &hash);

    // Generate a random payment hash for the verify URL
    let payment_hash: String = rand::thread_rng()
        .sample_iter(&rand::distributions::Alphanumeric)
        .take(64)
        .map(char::from)
        .collect::<String>()
        .to_lowercase();

    let verify_url = format!("https://{}/verify/{}/{}", state.domain, hash, payment_hash);

    Ok(Json(json!({
        "status": "OK",
        "pr": dummy_invoice,
        "verify": verify_url,
        "routes": [],
    })))
}

/// Computes the metadata string for a given name and domain.
pub fn calc_metadata(name: &str, domain: &str) -> String {
    format!("[[\"text/identifier\",\"{name}@{domain}\"],[\"text/plain\",\"Sats for {name}\"]]")
}

/// HTTP endpoint that provides the LNURL-pay metadata and parameters.
///
/// If the name is purely numeric (e.g., "123"), uses that as min/max sendable in satoshis.
/// Otherwise uses the default values from config.
pub async fn get_lnurl_pay(
    Path(name): Path<String>,
    Extension(state): Extension<State>,
) -> Result<Json<PayResponse>, (StatusCode, Json<Value>)> {
    if name.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "status": "ERROR",
                "reason": "Name parameter is required",
            })),
        ));
    }

    // Check if name is numeric - if so, use it as the amount in satoshis
    let (min_sendable, max_sendable) = if let Ok(sats) = name.parse::<u64>() {
        let msats = sats * 1000; // Convert sats to msats
        (msats, msats)
    } else {
        (state.min_sendable, state.max_sendable)
    };

    let metadata = calc_metadata(&name, &state.domain);
    let hash = sha256::Hash::hash(metadata.as_bytes());
    let callback = format!("https://{}/get-invoice/{}", state.domain, hex::encode(hash));

    let resp = PayResponse {
        callback,
        min_sendable,
        max_sendable,
        tag: Tag::PayRequest,
        metadata,
        comment_allowed: None,
        allows_nostr: Some(false),
        nostr_pubkey: None,
    };

    println!("LNURL request for {name}@{} - min/max: {} msats", state.domain, min_sendable);

    Ok(Json(resp))
}

/// HTTP endpoint for verifying invoice payment status.
/// Always returns settled: false since this is a dummy server.
pub async fn verify(
    Path((_desc_hash, _pay_hash)): Path<(String, String)>,
    Extension(_state): Extension<State>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    Ok(Json(json!({
        "status": "OK",
        "settled": false,
        "preimage": null,
        "pr": null,
    })))
}
