mod blockchain;
mod wallet;
mod api;

use actix_cors::Cors;
use actix_web::{web, App, HttpServer, middleware};
use std::sync::Mutex;

use api::AppState;
use blockchain::Blockchain;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("╔══════════════════════════════════════╗");
    println!("║        mini-web3  ·  Rust node        ║");
    println!("╚══════════════════════════════════════╝\n");

    // ── Quick CLI demo (runs before the server starts) ──────────────────────
    demo();

    // ── Start the HTTP node ──────────────────────────────────────────────────
    let data = web::Data::new(AppState {
        blockchain: Mutex::new(Blockchain::new()),
    });

    println!("\n  HTTP node listening on http://127.0.0.1:8080");
    println!("    Endpoints:");
    println!("      GET  /chain");
    println!("      GET  /balance/{{address}}");
    println!("      GET  /pending");
    println!("      GET  /validate");
    println!("      GET  /wallet/new");
    println!("      POST /transaction");
    println!("      POST /mine\n");

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            // CORS – allow any origin so a browser UI can call the node
            .wrap(Cors::permissive())
            .route("/chain",                web::get().to(api::get_chain))
            .route("/balance/{address}",    web::get().to(api::get_balance))
            .route("/pending",              web::get().to(api::get_pending))
            .route("/validate",             web::get().to(api::validate_chain))
            .route("/wallet/new",           web::get().to(api::new_wallet))
            .route("/transaction",          web::post().to(api::post_transaction))
            .route("/mine",                 web::post().to(api::mine_block))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}

/// Demonstrate wallet creation, transaction signing, and mining in the terminal
fn demo() {
    use blockchain::{Blockchain, Transaction};
    use chrono::Utc;
    use wallet::Wallet;

    println!("── Demo ────────────────────────────────\n");

    // 1. Create wallets
    let alice = Wallet::new();
    let bob   = Wallet::new();
    println!(" Alice");
    println!("    address : {}", alice.address);
    println!("    pub_key : {}\n", &alice.public_key[..16]);

    println!(" Bob");
    println!("    address : {}\n", bob.address);

    // 2. Boot the chain
    let mut bc = Blockchain::new();

    // 3. Mine to seed Alice with coins
    println!("⛏️   Mining genesis reward → Alice …");
    bc.mine_pending_transactions(&alice.address);
    println!("    Alice balance: {} coins\n", bc.balance_of(&alice.address));

    // 4. Alice signs a transaction to Bob
    let amount = 10.0_f64;
    let msg    = format!("{}{}{}", alice.address, bob.address, amount);
    let sig    = alice.sign(&msg).expect("signing failed");

    let tx = Transaction {
        id: uuid::Uuid::new_v4().to_string(),
        from: alice.address.clone(),
        to: bob.address.clone(),
        amount,
        timestamp: Utc::now().timestamp_millis(),
        signature: sig,
    };
    bc.add_transaction(tx);
    println!("  Alice → Bob  {} coins (signed & queued)", amount);

    // 5. Mine the pending transaction
    println!("   Mining block #1 …");
    bc.mine_pending_transactions(&alice.address);

    println!("\n  Balances after block #1:");
    println!("    Alice : {} coins", bc.balance_of(&alice.address));
    println!("    Bob   : {} coins", bc.balance_of(&bob.address));

    // 6. Validate
    println!("\n  Chain valid? {}", bc.is_valid());
    println!("    Blocks  : {}", bc.chain.len());
    println!("────────────────────────────────────────");
}
