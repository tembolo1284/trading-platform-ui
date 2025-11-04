use serde::{Deserialize, Serialize};
use std::net::SocketAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub matching_engine: MatchingEngineConfig,
    pub monte_carlo: MonteCarloConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Address to bind the gRPC server (e.g., "0.0.0.0:50051")
    pub bind_address: String,
    
    /// Enable gRPC-Web for browser clients
    pub enable_grpc_web: bool,
    
    /// Enable CORS for browser clients
    pub enable_cors: bool,
    
    /// Maximum concurrent connections
    pub max_connections: usize,
    
    /// Request timeout in seconds
    pub request_timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchingEngineConfig {
    /// TCP address of the matching engine gateway (e.g., "127.0.0.1:8080")
    pub gateway_address: String,
    
    /// Connection pool size
    pub pool_size: usize,
    
    /// Connection timeout in milliseconds
    pub connect_timeout_ms: u64,
    
    /// Read timeout in milliseconds
    pub read_timeout_ms: u64,
    
    /// Enable connection keep-alive
    pub keepalive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonteCarloConfig {
    /// Path to the Monte Carlo shared library
    pub library_path: String,
    
    /// Default number of simulations
    pub default_simulations: u64,
    
    /// Default number of time steps
    pub default_steps: u64,
    
    /// Enable antithetic variates by default
    pub default_antithetic: bool,
    
    /// Enable control variates by default
    pub default_control_variates: bool,
    
    /// Enable stratified sampling by default
    pub default_stratified_sampling: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                bind_address: "0.0.0.0:50051".to_string(),
                enable_grpc_web: true,
                enable_cors: true,
                max_connections: 1000,
                request_timeout_secs: 30,
            },
            matching_engine: MatchingEngineConfig {
                gateway_address: "127.0.0.1:8080".to_string(),
                pool_size: 10,
                connect_timeout_ms: 5000,
                read_timeout_ms: 10000,
                keepalive: true,
            },
            monte_carlo: MonteCarloConfig {
                library_path: "../MonteCarloLib/build/bin/release/libMonteCarloLib.so"
                    .to_string(),
                default_simulations: 10_000,
                default_steps: 252,
                default_antithetic: true,
                default_control_variates: false,
                default_stratified_sampling: false,
            },
        }
    }
}

impl Config {
    /// Load configuration from file or environment
    pub fn load() -> anyhow::Result<Self> {
        let config = config::Config::builder()
            .add_source(config::File::with_name("config").required(false))
            .add_source(config::Environment::with_prefix("TRADING"))
            .build()?;
        
        Ok(config.try_deserialize().unwrap_or_default())
    }
    
    /// Get the server socket address
    pub fn server_addr(&self) -> anyhow::Result<SocketAddr> {
        self.server
            .bind_address
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid bind address: {}", e))
    }
}
