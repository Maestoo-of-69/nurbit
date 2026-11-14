//! Nurbit medical server.
//!
//! Configuration (environment variables):
//! * `NURBIT_SYNC_TOKEN` - required access token (at least 16 characters);
//!   never written in code or files.
//! * `NURBIT_SERVER_DB`  - database file (default `nurbit-server.db`).
//! * `NURBIT_SERVER_ADDR`- listen address (default `127.0.0.1:8787`, local only).
//!
//! Plain HTTP is acceptable only on localhost for the demo; any other
//! deployment needs HTTPS (TLS) in front of it.

use nurbit_server::{MIN_TOKEN_LEN, build_app, open_database};

#[tokio::main]
async fn main() {
    let token = match std::env::var("NURBIT_SYNC_TOKEN") {
        Ok(t) if t.len() >= MIN_TOKEN_LEN => t,
        _ => {
            eprintln!("Set NURBIT_SYNC_TOKEN to a secret of at least {MIN_TOKEN_LEN} characters.");
            std::process::exit(2);
        }
    };
    let db = std::env::var("NURBIT_SERVER_DB").unwrap_or_else(|_| "nurbit-server.db".into());
    let addr = std::env::var("NURBIT_SERVER_ADDR").unwrap_or_else(|_| "127.0.0.1:8787".into());

    let pool = match open_database(&db).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Cannot open database {db}: {e}");
            std::process::exit(1);
        }
    };
    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Cannot listen on {addr}: {e}");
            std::process::exit(1);
        }
    };
    println!("Nurbit server listening on http://{addr} (database: {db})");
    println!("{}", nurbit_shared::SYNTHETIC_DATA_NOTICE);
    if let Err(e) = axum::serve(listener, build_app(pool, token)).await {
        eprintln!("Server stopped: {e}");
        std::process::exit(1);
    }
}
