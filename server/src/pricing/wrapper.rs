use super::ffi;
use crate::proto::pricing::{BarrierType, SimulationConfig};
use anyhow::Result;
use std::sync::Arc;
use parking_lot::Mutex;

/// Thread-safe wrapper around the Monte Carlo context
pub struct MonteCarloEngine {
    ctx: Arc<Mutex<MonteCarloContext>>,
}

struct MonteCarloContext {
    ptr: *mut ffi::mco_context_t,
}

impl MonteCarloContext {
    fn new() -> Result<Self> {
        let ptr = unsafe { ffi::mco_context_new() };
        if ptr.is_null() {
            anyhow::bail!("Failed to create Monte Carlo context");
        }
        Ok(Self { ptr })
    }
    
    fn configure(&mut self, config: &SimulationConfig) {
        unsafe {
            if config.seed > 0 {
                ffi::mco_context_set_seed(self.ptr, config.seed);
            }
            ffi::mco_context_set_num_simulations(self.ptr, config.num_simulations);
            ffi::mco_context_set_num_steps(self.ptr, config.num_steps);
            ffi::mco_context_set_antithetic(self.ptr, config.antithetic_enabled as i32);
            ffi::mco_context_set_control_variates(
                self.ptr,
                config.control_variates_enabled as i32,
            );
            ffi::mco_context_set_stratified_sampling(
                self.ptr,
                config.stratified_sampling_enabled as i32,
            );
        }
    }
}

impl Drop for MonteCarloContext {
    fn drop(&mut self) {
        if !self.ptr.is_null() {
            unsafe {
                ffi::mco_context_free(self.ptr);
            }
        }
    }
}

unsafe impl Send for MonteCarloContext {}

impl MonteCarloEngine {
    pub fn new() -> Result<Self> {
        let ctx = MonteCarloContext::new()?;
        Ok(Self {
            ctx: Arc::new(Mutex::new(ctx)),
        })
    }
    
    // European options
    pub fn price_european_call(
        &self,
        spot: f64,
        strike: f64,
        rate: f64,
        volatility: f64,
        time_to_maturity: f64,
        config: &SimulationConfig,
    ) -> f64 {
        let mut ctx = self.ctx.lock();
        ctx.configure(config);
        unsafe {
            ffi::mco_european_call(ctx.ptr, spot, strike, rate, volatility, time_to_maturity)
        }
    }
    
    pub fn price_european_put(
        &self,
        spot: f64,
        strike: f64,
        rate: f64,
        volatility: f64,
        time_to_maturity: f64,
        config: &SimulationConfig,
    ) -> f64 {
        let mut ctx = self.ctx.lock();
        ctx.configure(config);
        unsafe {
            ffi::mco_european_put(ctx.ptr, spot, strike, rate, volatility, time_to_maturity)
        }
    }
    
    // Asian options
    pub fn price_asian_call(
        &self,
        spot: f64,
        strike: f64,
        rate: f64,
        volatility: f64,
        time_to_maturity: f64,
        num_observations: u32,
        config: &SimulationConfig,
    ) -> f64 {
        let mut ctx = self.ctx.lock();
        ctx.configure(config);
        unsafe {
            ffi::mco_asian_arithmetic_call(
                ctx.ptr,
                spot,
                strike,
                rate,
                volatility,
                time_to_maturity,
                num_observations as usize,
            )
        }
    }
    
    pub fn price_asian_put(
        &self,
        spot: f64,
        strike: f64,
        rate: f64,
        volatility: f64,
        time_to_maturity: f64,
        num_observations: u32,
        config: &SimulationConfig,
    ) -> f64 {
        let mut ctx = self.ctx.lock();
        ctx.configure(config);
        unsafe {
            ffi::mco_asian_arithmetic_put(
                ctx.ptr,
                spot,
                strike,
                rate,
                volatility,
                time_to_maturity,
                num_observations as usize,
            )
        }
    }
    
    // American options
    pub fn price_american_call(
        &self,
        spot: f64,
        strike: f64,
        rate: f64,
        volatility: f64,
        time_to_maturity: f64,
        num_exercise_points: u32,
        config: &SimulationConfig,
    ) -> f64 {
        let mut ctx = self.ctx.lock();
        ctx.configure(config);
        unsafe {
            ffi::mco_american_call(
                ctx.ptr,
                spot,
                strike,
                rate,
                volatility,
                time_to_maturity,
                num_exercise_points as usize,
            )
        }
    }
    
    pub fn price_american_put(
        &self,
        spot: f64,
        strike: f64,
        rate: f64,
        volatility: f64,
        time_to_maturity: f64,
        num_exercise_points: u32,
        config: &SimulationConfig,
    ) -> f64 {
        let mut ctx = self.ctx.lock();
        ctx.configure(config);
        unsafe {
            ffi::mco_american_put(
                ctx.ptr,
                spot,
                strike,
                rate,
                volatility,
                time_to_maturity,
                num_exercise_points as usize,
            )
        }
    }
    // Bermudan options
    pub fn price_bermudan_call(
        &self,
        spot: f64,
        strike: f64,
        rate: f64,
        volatility: f64,
        exercise_dates: &[f64],
        config: &SimulationConfig,
    ) -> f64 {
        let mut ctx = self.ctx.lock();
        ctx.configure(config);
        unsafe {
            ffi::mco_bermudan_call(
                ctx.ptr,
                spot,
                strike,
                rate,
                volatility,
                exercise_dates.as_ptr(),
                exercise_dates.len(),
            )
        }
    }
    
    pub fn price_bermudan_put(
        &self,
        spot: f64,
        strike: f64,
        rate: f64,
        volatility: f64,
        exercise_dates: &[f64],
        config: &SimulationConfig,
    ) -> f64 {
        let mut ctx = self.ctx.lock();
        ctx.configure(config);
        unsafe {
            ffi::mco_bermudan_put(
                ctx.ptr,
                spot,
                strike,
                rate,
                volatility,
                exercise_dates.as_ptr(),
                exercise_dates.len(),
            )
        }
    }
    
    // Barrier options
    pub fn price_barrier_call(
        &self,
        spot: f64,
        strike: f64,
        rate: f64,
        volatility: f64,
        time_to_maturity: f64,
        barrier_level: f64,
        barrier_type: BarrierType,
        rebate: f64,
        config: &SimulationConfig,
    ) -> f64 {
        let mut ctx = self.ctx.lock();
        ctx.configure(config);
        unsafe {
            ffi::mco_barrier_call(
                ctx.ptr,
                spot,
                strike,
                rate,
                volatility,
                time_to_maturity,
                barrier_level,
                barrier_type as i32,
                rebate,
            )
        }
    }
    
    pub fn price_barrier_put(
        &self,
        spot: f64,
        strike: f64,
        rate: f64,
        volatility: f64,
        time_to_maturity: f64,
        barrier_level: f64,
        barrier_type: BarrierType,
        rebate: f64,
        config: &SimulationConfig,
    ) -> f64 {
        let mut ctx = self.ctx.lock();
        ctx.configure(config);
        unsafe {
            ffi::mco_barrier_put(
                ctx.ptr,
                spot,
                strike,
                rate,
                volatility,
                time_to_maturity,
                barrier_level,
                barrier_type as i32,
                rebate,
            )
        }
    }
    
    // Lookback options
    pub fn price_lookback_call(
        &self,
        spot: f64,
        strike: f64,
        rate: f64,
        volatility: f64,
        time_to_maturity: f64,
        fixed_strike: bool,
        config: &SimulationConfig,
    ) -> f64 {
        let mut ctx = self.ctx.lock();
        ctx.configure(config);
        unsafe {
            ffi::mco_lookback_call(
                ctx.ptr,
                spot,
                strike,
                rate,
                volatility,
                time_to_maturity,
                fixed_strike as i32,
            )
        }
    }
    
    pub fn price_lookback_put(
        &self,
        spot: f64,
        strike: f64,
        rate: f64,
        volatility: f64,
        time_to_maturity: f64,
        fixed_strike: bool,
        config: &SimulationConfig,
    ) -> f64 {
        let mut ctx = self.ctx.lock();
        ctx.configure(config);
        unsafe {
            ffi::mco_lookback_put(
                ctx.ptr,
                spot,
                strike,
                rate,
                volatility,
                time_to_maturity,
                fixed_strike as i32,
            )
        }
    }
}

impl Clone for MonteCarloEngine {
    fn clone(&self) -> Self {
        Self {
            ctx: Arc::clone(&self.ctx),
        }
    }
}
