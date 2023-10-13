pub mod config;
pub mod error;
mod rpc;
pub mod types;
mod utils;

use axum::routing::post;
use axum::{routing::get, Router};
use config::{config, Config};
use ethers::providers::{Http, Provider};
use hyper::Method;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utils::{load_provider, load_signer};

pub async fn run() {
    // Setup tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "paymaster=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Setup config
    let c = config();

    // Setup app state
    let app_state = Arc::new(AppState::load_from_config(c));

    // Setup CORS
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST])
        .allow_origin(Any);

    // Setup routes
    let route = axum::Router::new().route("/health_check", get(|| async { "OK" }));
    let rpc_route = axum::Router::new()
        .route("/", post(rpc::handle_rpc_request))
        .layer(cors);
    let router = Router::new()
        .nest("/rpc", rpc_route)
        .nest("/", route)
        .with_state(app_state);

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], c.PORT));
    axum::Server::bind(&addr)
        .serve(router.into_make_service())
        .await
        .unwrap();
}

pub struct AppState {
    pub signer: ethers::signers::LocalWallet,
    pub mumbai_provider: Provider<Http>,
    pub sepolia_provider: Provider<Http>,
    pub goerli_provider: Provider<Http>,
}

impl AppState {
    pub fn load_from_config(config: &Config) -> Self {
        Self {
            signer: load_signer(config),
            mumbai_provider: load_provider(config, "mumbai"),
            sepolia_provider: load_provider(config, "sepolia"),
            goerli_provider: load_provider(config, "goerli"),
        }
    }
}
