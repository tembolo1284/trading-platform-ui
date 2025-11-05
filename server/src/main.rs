mod config;
mod matching;
mod pricing;
mod proto;
mod services;

use crate::config::Config;
use crate::matching::MatchingClient;
use crate::pricing::MonteCarloEngine;
use crate::proto::pricing::pricing_service_server::PricingServiceServer;
use crate::proto::trading::trading_service_server::TradingServiceServer;
use crate::services::{PricingServiceImpl, TradingServiceImpl};

use anyhow::{Context, Result};
use std::sync::Arc;
use tonic::transport::Server;
use tonic_web::GrpcWebLayer;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "trading_server=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting Trading Platform gRPC Server");

    // Load configuration
    let config = Config::load().context("Failed to load configuration")?;
    info!("Configuration loaded: {:#?}", config);

    // Initialize Monte Carlo engine
    info!(
        "Initializing Monte Carlo engine from: {}",
        config.monte_carlo.library_path
    );
    let monte_carlo_engine = Arc::new(
        MonteCarloEngine::new().context("Failed to initialize Monte Carlo engine")?,
    );
    info!("Monte Carlo engine initialized");

    // Initialize matching engine client
    info!(
        "Connecting to matching engine at: {}",
        config.matching_engine.gateway_address
    );
    let matching_client = Arc::new(
        MatchingClient::new(
            config.matching_engine.gateway_address.clone(),
            config.matching_engine.pool_size,
            config.matching_engine.connect_timeout_ms,
        )
        .await
        .context("Failed to connect to matching engine")?,
    );
    info!("Connected to matching engine");

    // Create gRPC services
    let pricing_service = PricingServiceImpl::new(Arc::clone(&monte_carlo_engine));
    let trading_service = TradingServiceImpl::new(Arc::clone(&matching_client));

    // Get server address
    let addr = config
        .server_addr()
        .context("Failed to parse server address")?;

    info!("gRPC server listening on {}", addr);

    // Build server - only gRPC-Web for now (tower-http CORS has compatibility issues)
    if config.server.enable_cors {
        warn!("CORS via tower-http has compatibility issues - skipping for now");
        warn!("gRPC-Web provides necessary browser support");
    }

    let result = if config.server.enable_grpc_web {
        info!("Enabling gRPC-Web for browser support");
        Server::builder()
            .accept_http1(true)
            .layer(GrpcWebLayer::new())
            .add_service(PricingServiceServer::new(pricing_service))
            .add_service(TradingServiceServer::new(trading_service))
            .serve(addr)
            .await
    } else {
        info!("Running in gRPC-only mode (no browser support)");
        Server::builder()
            .add_service(PricingServiceServer::new(pricing_service))
            .add_service(TradingServiceServer::new(trading_service))
            .serve(addr)
            .await
    };

    info!("Server started successfully!");
    info!("");
    info!("Available services:");
    info!("  - pricing.PricingService (Monte Carlo options pricing)");
    info!("  - trading.TradingService (Order submission and market data)");
    info!("");
    info!("Server is ready to accept connections");

    // Handle result
    if let Err(e) = result {
        error!("Server error: {}", e);
        return Err(e.into());
    }

    Ok(())
}
