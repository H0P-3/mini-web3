use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use chrono::Utc;
use uuid::Uuid;

use crate::blockchain::{Blockchain, Transaction};
use crate::wallet::{Wallet, verify_signature};

// ── Shared state ──────────────────────────────────────────────────────────────

pub struct AppState {
    pub blockchain: Mutex<Blockchain>,
}

// ── Request / Response shapes ─────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct SendTxRequest {
    pub from: String,
    pub to: String,
    pub amount: f64,
    pub public_key: String,
    pub signature: String,
}

#[derive(Deserialize)]
pub struct MineRequest {
    pub miner_address: String,
}

#[derive(Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: T,
}

fn ok<T: Serialize>(data: T) -> HttpResponse {
    HttpResponse::Ok().json(ApiResponse { success: true, data })
}

fn err(msg: &str) -> HttpResponse {
    HttpResponse::BadRequest().json(serde_json::json!({
        "success": false,
        "error": msg
    }))
}

// ── Handlers ──────────────────────────────────────────────────────────────────

/// GET /chain  →  the full blockchain
pub async fn get_chain(state: web::Data<AppState>) -> impl Responder {
    let bc = state.blockchain.lock().unwrap();
    ok(&*bc)
}

/// GET /balance/{address}  →  confirmed balance
pub async fn get_balance(
    state: web::Data<AppState>,
    path: web::Path<String>,
) -> impl Responder {
    let address = path.into_inner();
    let bc = state.blockchain.lock().unwrap();
    let balance = bc.balance_of(&address);
    ok(serde_json::json!({ "address": address, "balance": balance }))
}

/// GET /pending  →  mempool
pub async fn get_pending(state: web::Data<AppState>) -> impl Responder {
    let bc = state.blockchain.lock().unwrap();
    ok(&bc.pending_transactions)
}

/// POST /transaction  →  add a signed transaction to the mempool
pub async fn post_transaction(
    state: web::Data<AppState>,
    body: web::Json<SendTxRequest>,
) -> impl Responder {
    // Reconstruct the message the client signed
    let msg = format!("{}{}{}", body.from, body.to, body.amount);

    if !verify_signature(&body.public_key, &msg, &body.signature) {
        return err("Invalid signature – transaction rejected");
    }

    let tx = Transaction {
        id: Uuid::new_v4().to_string(),
        from: body.from.clone(),
        to: body.to.clone(),
        amount: body.amount,
        timestamp: Utc::now().timestamp_millis(),
        signature: body.signature.clone(),
    };

    let mut bc = state.blockchain.lock().unwrap();
    bc.add_transaction(tx.clone());
    ok(tx)
}

/// POST /mine  →  mine pending transactions into a new block
pub async fn mine_block(
    state: web::Data<AppState>,
    body: web::Json<MineRequest>,
) -> impl Responder {
    let miner = body.miner_address.clone();
    if miner.is_empty() {
        return err("miner_address is required");
    }

    let mut bc = state.blockchain.lock().unwrap();
    let block = bc.mine_pending_transactions(&miner);
    ok(block)
}

/// GET /validate  →  check the chain's integrity
pub async fn validate_chain(state: web::Data<AppState>) -> impl Responder {
    let bc = state.blockchain.lock().unwrap();
    let valid = bc.is_valid();
    ok(serde_json::json!({
        "valid": valid,
        "blocks": bc.chain.len()
    }))
}

/// GET /wallet/new  →  generate a fresh wallet
pub async fn new_wallet() -> impl Responder {
    let wallet = Wallet::new();
    ok(wallet)
}
