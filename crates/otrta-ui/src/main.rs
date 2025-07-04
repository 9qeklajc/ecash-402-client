use axum::{
    Router,
    routing::{delete, get, post},
};
mod background;
mod connection;
use background::BackgroundJobRunner;
use connection::{DatabaseSettings, Settings, get_configuration};
use ecash_402_wallet::wallet::CashuWalletClient;
use otrta::{
    db::server_config::create_with_seed,
    handlers::{self, get_server_config},
    models::AppState,
    proxy::{forward_any_request, forward_any_request_get},
};
use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "ecash-402-wallet=debug,tower_http=warning".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool = get_connection_pool(&configuration.database)
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .unwrap();

    let wallet = initialize_wallet(&connection_pool, &configuration, "ecash_402")
        .await
        .unwrap();

    let app_state = Arc::new(AppState {
        db: connection_pool.clone(),
        default_msats_per_request: configuration.application.default_msats_per_request,
        wallet,
    });

    let job_runner = BackgroundJobRunner::new(Arc::clone(&app_state));
    job_runner.start_all_jobs().await;

    let app = Router::new()
        .route("/api/openai-models", get(handlers::list_openai_models))
        .route("/api/proxy/models", get(handlers::get_proxy_models))
        .route("/api/providers", get(handlers::get_providers))
        .route(
            "/api/providers",
            post(handlers::create_custom_provider_handler),
        )
        .route(
            "/api/providers/default",
            get(handlers::get_default_provider_handler),
        )
        .route("/api/providers/{id}", get(handlers::get_provider))
        .route(
            "/api/providers/{id}",
            delete(handlers::delete_custom_provider_handler),
        )
        .route(
            "/api/providers/{id}/set-default",
            post(handlers::set_provider_default),
        )
        .route("/api/providers/refresh", post(handlers::refresh_providers))
        .route(
            "/api/proxy/models/refresh",
            post(handlers::refresh_models_from_proxy),
        )
        .route(
            "/api/wallet/redeem-pendings",
            post(handlers::redeem_pendings),
        )
        .route("/api/wallet/balance", get(handlers::get_balance))
        .route("/api/wallet/redeem", post(handlers::redeem_token))
        .route("/api/wallet/send", post(handlers::send_token))
        .route(
            "/api/wallet/pending-transactions",
            get(handlers::get_pendings),
        )
        .route(
            "/api/server-config",
            get(handlers::get_current_server_config),
        )
        .route("/api/server-config", post(handlers::update_server_config))
        .route("/api/credits", get(handlers::get_all_credits))
        .route("/api/transactions", get(handlers::get_all_transactions))
        .route("/{*path}", post(forward_any_request))
        .route("/v1/{*path}", post(forward_any_request))
        .route("/{*path}", get(forward_any_request_get))
        .route("/v1/{*path}", get(forward_any_request_get))
        .with_state(app_state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any)
                .expose_headers(Any)
                .allow_private_network(true),
        )
        .layer(TraceLayer::new_for_http());
    println!(
        "Server starting on http://{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = tokio::net::TcpListener::bind(format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    ))
    .await
    .unwrap();
    axum::serve(listener, app).await.unwrap();
}

pub async fn get_connection_pool(configuration: &DatabaseSettings) -> Result<PgPool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(configuration.connections)
        .connect_with(configuration.with_db())
        .await
}

async fn initialize_wallet(
    connection_pool: &PgPool,
    configuration: &Settings,
    db_name: &str,
) -> Result<CashuWalletClient, Box<dyn std::error::Error>> {
    let wallet_dir = dotenv::var("WALLET_DATA_DIR").unwrap_or_else(|_| "./wallet_data".to_string());
    std::fs::create_dir_all(&wallet_dir)?;

    let unique_db_name = format!("{}/{}", wallet_dir, db_name);
    let config = get_server_config(connection_pool).await;
    match config {
        Some(config) => Ok(CashuWalletClient::from_seed(
            &configuration.application.mint_url,
            &config.seed.clone().unwrap(),
            &unique_db_name,
        )
        .await
        .unwrap()),
        None => {
            let mut seed = String::new();
            let wallet = CashuWalletClient::new(
                &configuration.application.mint_url,
                &mut seed,
                &unique_db_name,
            )
            .await
            .unwrap();

            create_with_seed(connection_pool, &seed).await?;
            Ok(wallet)
        }
    }
}
