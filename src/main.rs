use axum::http::{Method, StatusCode, Uri};
use axum::routing::get;
use axum::{http, Extension, Json, Router};
use clap::Parser;
use serde::Serialize;
use tower_http::cors::{Any, CorsLayer};

use crate::config::*;
use crate::routes::*;

mod config;
mod routes;

#[derive(Clone)]
pub struct State {
    pub domain: String,
    pub min_sendable: u64,
    pub max_sendable: u64,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config: Config = Config::parse();

    let state = State {
        domain: config.domain.clone(),
        min_sendable: config.min_sendable,
        max_sendable: config.max_sendable,
    };

    let addr: std::net::SocketAddr = format!("{}:{}", config.bind, config.port)
        .parse()
        .expect("Failed to parse bind/port for webserver");

    println!("Dummy LNURL server running on http://{}", addr);
    println!("Domain: {}", config.domain);

    let server_router = Router::new()
        .route("/health-check", get(health_check))
        .route("/get-invoice/:hash", get(get_invoice))
        .route("/verify/:desc_hash/:pay_hash", get(verify))
        .route("/.well-known/lnurlp/:name", get(get_lnurl_pay))
        .fallback(fallback)
        .layer(Extension(state))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_headers(vec![http::header::CONTENT_TYPE])
                .allow_methods([Method::GET, Method::POST]),
        );

    let server = axum::Server::bind(&addr).serve(server_router.into_make_service());

    let graceful = server.with_graceful_shutdown(async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to create Ctrl+C shutdown signal");
    });

    if let Err(e) = graceful.await {
        eprintln!("shutdown error: {}", e);
    }

    Ok(())
}

async fn fallback(uri: Uri) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, format!("No route for {}", uri))
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

impl HealthResponse {
    pub fn new_ok() -> Self {
        Self {
            status: String::from("pass"),
            version: String::from("0"),
        }
    }
}

pub async fn health_check() -> Result<Json<HealthResponse>, (StatusCode, String)> {
    Ok(Json(HealthResponse::new_ok()))
}
