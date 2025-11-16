use std::{net::SocketAddr, sync::Arc};

use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use dotenvy::dotenv;
use tokio::signal;
use tracing::info;
use tracing_subscriber::{fmt, EnvFilter};

mod kraken;
mod models;

use kraken::{build_client_from_env, KrakenClient, KrakenClientError};
use models::{
    AddOrderRequest, AddOrderResponse, CancelOrderResponse, QueryOrderQuery, QueryOrdersResponse,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    init_tracing();

    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(8080);

    let client = Arc::new(build_client_from_env()?);

    let app = Router::new()
        .route("/orders", post(create_order))
        .route("/orders/:txid", get(get_order).delete(cancel_order))
        .with_state(client);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("listening on {addr}");
    axum::serve(tokio::net::TcpListener::bind(addr).await?, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn create_order(
    State(client): State<Arc<KrakenClient>>,
    Json(payload): Json<AddOrderRequest>,
) -> Result<Json<AddOrderResponse>, KrakenClientError> {
    let response = client.add_order(payload).await?;
    Ok(Json(response))
}

async fn cancel_order(
    State(client): State<Arc<KrakenClient>>,
    Path(txid): Path<String>,
) -> Result<Json<CancelOrderResponse>, KrakenClientError> {
    let response = client.cancel_order(&txid).await?;
    Ok(Json(response))
}

async fn get_order(
    State(client): State<Arc<KrakenClient>>,
    Path(txid): Path<String>,
    Query(params): Query<QueryOrderQuery>,
) -> Result<Json<QueryOrdersResponse>, KrakenClientError> {
    let response = client.query_order(&txid, params.trades).await?;
    Ok(Json(response))
}

fn init_tracing() {
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    fmt().with_env_filter(env_filter).init();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
