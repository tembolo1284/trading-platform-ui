use crate::pricing::MonteCarloEngine;
use crate::proto::pricing::{
    pricing_service_server::PricingService, AmericanRequest, AsianRequest, BarrierRequest,
    BatchRequest, BatchResponse, BermudanRequest, EuropeanRequest, LookbackRequest,
    MarketPriceRequest, PriceResponse, SimulationConfig,
};
use std::sync::Arc;
use std::time::Instant;
use tonic::{Request, Response, Status};
use tracing::{debug, error, info};

/// Pricing service implementation
#[derive(Clone)]
pub struct PricingServiceImpl {
    engine: Arc<MonteCarloEngine>,
}

impl PricingServiceImpl {
    pub fn new(engine: Arc<MonteCarloEngine>) -> Self {
        Self { engine }
    }
    
    /// Get config with defaults if not provided
    fn get_config(config: Option<SimulationConfig>) -> SimulationConfig {
        config.unwrap_or_else(|| SimulationConfig {
            num_simulations: 10_000,
            num_steps: 252,
            seed: 0,
            antithetic_enabled: true,
            control_variates_enabled: false,
            stratified_sampling_enabled: false,
        })
    }
}

#[tonic::async_trait]
impl PricingService for PricingServiceImpl {
    async fn price_european_call(
        &self,
        request: Request<EuropeanRequest>,
    ) -> Result<Response<PriceResponse>, Status> {
        let req = request.into_inner();
        let config = Self::get_config(req.config);
        
        debug!(
            "Pricing European call: spot={}, strike={}, ttm={}",
            req.spot, req.strike, req.time_to_maturity
        );
        
        let start = Instant::now();
        
        let price = self.engine.price_european_call(
            req.spot,
            req.strike,
            req.rate,
            req.volatility,
            req.time_to_maturity,
            &config,
        );
        
        let computation_time_ms = start.elapsed().as_secs_f64() * 1000.0;
        
        info!(
            "European call priced: ${:.4} in {:.2}ms",
            price, computation_time_ms
        );
        
        Ok(Response::new(PriceResponse {
            price,
            computation_time_ms,
            error_message: String::new(),
            delta: None,
            gamma: None,
            vega: None,
            theta: None,
            rho: None,
        }))
    }
    
    async fn price_european_put(
        &self,
        request: Request<EuropeanRequest>,
    ) -> Result<Response<PriceResponse>, Status> {
        let req = request.into_inner();
        let config = Self::get_config(req.config);
        
        debug!(
            "Pricing European put: spot={}, strike={}, ttm={}",
            req.spot, req.strike, req.time_to_maturity
        );
        
        let start = Instant::now();
        
        let price = self.engine.price_european_put(
            req.spot,
            req.strike,
            req.rate,
            req.volatility,
            req.time_to_maturity,
            &config,
        );
        
        let computation_time_ms = start.elapsed().as_secs_f64() * 1000.0;
        
        info!(
            "European put priced: ${:.4} in {:.2}ms",
            price, computation_time_ms
        );
        
        Ok(Response::new(PriceResponse {
            price,
            computation_time_ms,
            error_message: String::new(),
            delta: None,
            gamma: None,
            vega: None,
            theta: None,
            rho: None,
        }))
    }
    
    async fn price_american_call(
        &self,
        request: Request<AmericanRequest>,
    ) -> Result<Response<PriceResponse>, Status> {
        let req = request.into_inner();
        let config = Self::get_config(req.config);
        
        let start = Instant::now();
        
        let price = self.engine.price_american_call(
            req.spot,
            req.strike,
            req.rate,
            req.volatility,
            req.time_to_maturity,
            req.num_exercise_points,
            &config,
        );
        
        let computation_time_ms = start.elapsed().as_secs_f64() * 1000.0;
        
        Ok(Response::new(PriceResponse {
            price,
            computation_time_ms,
            error_message: String::new(),
            delta: None,
            gamma: None,
            vega: None,
            theta: None,
            rho: None,
        }))
    }
    
    async fn price_american_put(
        &self,
        request: Request<AmericanRequest>,
    ) -> Result<Response<PriceResponse>, Status> {
        let req = request.into_inner();
        let config = Self::get_config(req.config);
        
        let start = Instant::now();
        
        let price = self.engine.price_american_put(
            req.spot,
            req.strike,
            req.rate,
            req.volatility,
            req.time_to_maturity,
            req.num_exercise_points,
            &config,
        );
        
        let computation_time_ms = start.elapsed().as_secs_f64() * 1000.0;
        
        Ok(Response::new(PriceResponse {
            price,
            computation_time_ms,
            error_message: String::new(),
            delta: None,
            gamma: None,
            vega: None,
            theta: None,
            rho: None,
        }))
    }
    
    async fn price_asian_call(
        &self,
        request: Request<AsianRequest>,
    ) -> Result<Response<PriceResponse>, Status> {
        let req = request.into_inner();
        let config = Self::get_config(req.config);
        
        let start = Instant::now();
        
        let price = self.engine.price_asian_call(
            req.spot,
            req.strike,
            req.rate,
            req.volatility,
            req.time_to_maturity,
            req.num_observations,
            &config,
        );
        
        let computation_time_ms = start.elapsed().as_secs_f64() * 1000.0;
        
        Ok(Response::new(PriceResponse {
            price,
            computation_time_ms,
            error_message: String::new(),
            delta: None,
            gamma: None,
            vega: None,
            theta: None,
            rho: None,
        }))
    }
    
    async fn price_asian_put(
        &self,
        request: Request<AsianRequest>,
    ) -> Result<Response<PriceResponse>, Status> {
        let req = request.into_inner();
        let config = Self::get_config(req.config);
        
        let start = Instant::now();
        
        let price = self.engine.price_asian_put(
            req.spot,
            req.strike,
            req.rate,
            req.volatility,
            req.time_to_maturity,
            req.num_observations,
            &config,
        );
        
        let computation_time_ms = start.elapsed().as_secs_f64() * 1000.0;
        
        Ok(Response::new(PriceResponse {
            price,
            computation_time_ms,
            error_message: String::new(),
            delta: None,
            gamma: None,
            vega: None,
            theta: None,
            rho: None,
        }))
    }
async fn price_barrier_call(
        &self,
        request: Request<BarrierRequest>,
    ) -> Result<Response<PriceResponse>, Status> {
        let req = request.into_inner();
        let config = Self::get_config(req.config);
        
        let start = Instant::now();
        
        let barrier_type = crate::proto::pricing::BarrierType::try_from(req.barrier_type)
            .map_err(|_| Status::invalid_argument("Invalid barrier type"))?;
        
        let price = self.engine.price_barrier_call(
            req.spot,
            req.strike,
            req.rate,
            req.volatility,
            req.time_to_maturity,
            req.barrier_level,
            barrier_type,
            req.rebate,
            &config,
        );
        
        let computation_time_ms = start.elapsed().as_secs_f64() * 1000.0;
        
        Ok(Response::new(PriceResponse {
            price,
            computation_time_ms,
            error_message: String::new(),
            delta: None,
            gamma: None,
            vega: None,
            theta: None,
            rho: None,
        }))
    }
    
    async fn price_barrier_put(
        &self,
        request: Request<BarrierRequest>,
    ) -> Result<Response<PriceResponse>, Status> {
        let req = request.into_inner();
        let config = Self::get_config(req.config);
        
        let start = Instant::now();
        
        let barrier_type = crate::proto::pricing::BarrierType::try_from(req.barrier_type)
            .map_err(|_| Status::invalid_argument("Invalid barrier type"))?;
        
        let price = self.engine.price_barrier_put(
            req.spot,
            req.strike,
            req.rate,
            req.volatility,
            req.time_to_maturity,
            req.barrier_level,
            barrier_type,
            req.rebate,
            &config,
        );
        
        let computation_time_ms = start.elapsed().as_secs_f64() * 1000.0;
        
        Ok(Response::new(PriceResponse {
            price,
            computation_time_ms,
            error_message: String::new(),
            delta: None,
            gamma: None,
            vega: None,
            theta: None,
            rho: None,
        }))
    }
    
    async fn price_lookback_call(
        &self,
        request: Request<LookbackRequest>,
    ) -> Result<Response<PriceResponse>, Status> {
        let req = request.into_inner();
        let config = Self::get_config(req.config);
        
        let start = Instant::now();
        
        let price = self.engine.price_lookback_call(
            req.spot,
            req.strike,
            req.rate,
            req.volatility,
            req.time_to_maturity,
            req.fixed_strike,
            &config,
        );
        
        let computation_time_ms = start.elapsed().as_secs_f64() * 1000.0;
        
        Ok(Response::new(PriceResponse {
            price,
            computation_time_ms,
            error_message: String::new(),
            delta: None,
            gamma: None,
            vega: None,
            theta: None,
            rho: None,
        }))
    }
    
    async fn price_lookback_put(
        &self,
        request: Request<LookbackRequest>,
    ) -> Result<Response<PriceResponse>, Status> {
        let req = request.into_inner();
        let config = Self::get_config(req.config);
        
        let start = Instant::now();
        
        let price = self.engine.price_lookback_put(
            req.spot,
            req.strike,
            req.rate,
            req.volatility,
            req.time_to_maturity,
            req.fixed_strike,
            &config,
        );
        
        let computation_time_ms = start.elapsed().as_secs_f64() * 1000.0;
        
        Ok(Response::new(PriceResponse {
            price,
            computation_time_ms,
            error_message: String::new(),
            delta: None,
            gamma: None,
            vega: None,
            theta: None,
            rho: None,
        }))
    }
    
    async fn price_bermudan_call(
        &self,
        request: Request<BermudanRequest>,
    ) -> Result<Response<PriceResponse>, Status> {
        let req = request.into_inner();
        let config = Self::get_config(req.config);
        
        let start = Instant::now();
        
        let price = self.engine.price_bermudan_call(
            req.spot,
            req.strike,
            req.rate,
            req.volatility,
            &req.exercise_dates,
            &config,
        );
        
        let computation_time_ms = start.elapsed().as_secs_f64() * 1000.0;
        
        Ok(Response::new(PriceResponse {
            price,
            computation_time_ms,
            error_message: String::new(),
            delta: None,
            gamma: None,
            vega: None,
            theta: None,
            rho: None,
        }))
    }
    
    async fn price_bermudan_put(
        &self,
        request: Request<BermudanRequest>,
    ) -> Result<Response<PriceResponse>, Status> {
        let req = request.into_inner();
        let config = Self::get_config(req.config);
        
        let start = Instant::now();
        
        let price = self.engine.price_bermudan_put(
            req.spot,
            req.strike,
            req.rate,
            req.volatility,
            &req.exercise_dates,
            &config,
        );
        
        let computation_time_ms = start.elapsed().as_secs_f64() * 1000.0;
        
        Ok(Response::new(PriceResponse {
            price,
            computation_time_ms,
            error_message: String::new(),
            delta: None,
            gamma: None,
            vega: None,
            theta: None,
            rho: None,
        }))
    }
async fn price_batch(
        &self,
        request: Request<BatchRequest>,
    ) -> Result<Response<BatchResponse>, Status> {
        let req = request.into_inner();
        let config = Self::get_config(req.config);
        
        let start = Instant::now();
        
        let mut call_prices = Vec::new();
        let mut put_prices = Vec::new();
        
        // Price all calls
        for call_req in req.european_calls {
            let price = self.engine.price_european_call(
                call_req.spot,
                call_req.strike,
                call_req.rate,
                call_req.volatility,
                call_req.time_to_maturity,
                &config,
            );
            call_prices.push(price);
        }
        
        // Price all puts
        for put_req in req.european_puts {
            let price = self.engine.price_european_put(
                put_req.spot,
                put_req.strike,
                put_req.rate,
                put_req.volatility,
                put_req.time_to_maturity,
                &config,
            );
            put_prices.push(price);
        }
        
        let total_computation_time_ms = start.elapsed().as_secs_f64() * 1000.0;
        
        info!(
            "Batch priced: {} calls + {} puts in {:.2}ms",
            call_prices.len(),
            put_prices.len(),
            total_computation_time_ms
        );
        
        Ok(Response::new(BatchResponse {
            european_call_prices: call_prices,
            european_put_prices: put_prices,
            total_computation_time_ms,
        }))
    }
    
    async fn price_from_market(
        &self,
        request: Request<MarketPriceRequest>,
    ) -> Result<Response<PriceResponse>, Status> {
        let _req = request.into_inner();
        
        // TODO: Implement market data fetching
        // This would query the order book for current spot price
        // and potentially estimate volatility from recent trades
        
        Err(Status::unimplemented(
            "Market-based pricing not yet implemented",
        ))
    }
}
