use libc::{c_double, c_int, size_t, uint64_t};

// Opaque context type
#[repr(C)]
pub struct mco_context_t {
    _private: [u8; 0],
}

// FFI declarations matching mcoptions.h
extern "C" {
    // Context management
    pub fn mco_context_new() -> *mut mco_context_t;
    pub fn mco_context_free(ctx: *mut mco_context_t);
    
    // Configuration
    pub fn mco_context_set_seed(ctx: *mut mco_context_t, seed: uint64_t);
    pub fn mco_context_set_num_simulations(ctx: *mut mco_context_t, n: uint64_t);
    pub fn mco_context_set_num_steps(ctx: *mut mco_context_t, n: uint64_t);
    pub fn mco_context_set_antithetic(ctx: *mut mco_context_t, enabled: c_int);
    pub fn mco_context_set_control_variates(ctx: *mut mco_context_t, enabled: c_int);
    pub fn mco_context_set_stratified_sampling(ctx: *mut mco_context_t, enabled: c_int);
    pub fn mco_context_set_importance_sampling(
        ctx: *mut mco_context_t,
        enabled: c_int,
        drift_shift: c_double,
    );
    
    // European options
    pub fn mco_european_call(
        ctx: *mut mco_context_t,
        spot: c_double,
        strike: c_double,
        rate: c_double,
        volatility: c_double,
        time_to_maturity: c_double,
    ) -> c_double;
    
    pub fn mco_european_put(
        ctx: *mut mco_context_t,
        spot: c_double,
        strike: c_double,
        rate: c_double,
        volatility: c_double,
        time_to_maturity: c_double,
    ) -> c_double;
    
    // Asian options
    pub fn mco_asian_arithmetic_call(
        ctx: *mut mco_context_t,
        spot: c_double,
        strike: c_double,
        rate: c_double,
        volatility: c_double,
        time_to_maturity: c_double,
        num_observations: size_t,
    ) -> c_double;
    
    pub fn mco_asian_arithmetic_put(
        ctx: *mut mco_context_t,
        spot: c_double,
        strike: c_double,
        rate: c_double,
        volatility: c_double,
        time_to_maturity: c_double,
        num_observations: size_t,
    ) -> c_double;
    
    // American options
    pub fn mco_american_call(
        ctx: *mut mco_context_t,
        spot: c_double,
        strike: c_double,
        rate: c_double,
        volatility: c_double,
        time_to_maturity: c_double,
        num_exercise_points: size_t,
    ) -> c_double;
    
    pub fn mco_american_put(
        ctx: *mut mco_context_t,
        spot: c_double,
        strike: c_double,
        rate: c_double,
        volatility: c_double,
        time_to_maturity: c_double,
        num_exercise_points: size_t,
    ) -> c_double;
    
    // Bermudan options
    pub fn mco_bermudan_call(
        ctx: *mut mco_context_t,
        spot: c_double,
        strike: c_double,
        rate: c_double,
        volatility: c_double,
        exercise_dates: *const c_double,
        num_dates: size_t,
    ) -> c_double;
    
    pub fn mco_bermudan_put(
        ctx: *mut mco_context_t,
        spot: c_double,
        strike: c_double,
        rate: c_double,
        volatility: c_double,
        exercise_dates: *const c_double,
        num_dates: size_t,
    ) -> c_double;
    
    // Barrier options
    pub fn mco_barrier_call(
        ctx: *mut mco_context_t,
        spot: c_double,
        strike: c_double,
        rate: c_double,
        volatility: c_double,
        time_to_maturity: c_double,
        barrier_level: c_double,
        barrier_type: c_int,
        rebate: c_double,
    ) -> c_double;
    
    pub fn mco_barrier_put(
        ctx: *mut mco_context_t,
        spot: c_double,
        strike: c_double,
        rate: c_double,
        volatility: c_double,
        time_to_maturity: c_double,
        barrier_level: c_double,
        barrier_type: c_int,
        rebate: c_double,
    ) -> c_double;
    
    // Lookback options
    pub fn mco_lookback_call(
        ctx: *mut mco_context_t,
        spot: c_double,
        strike: c_double,
        rate: c_double,
        volatility: c_double,
        time_to_maturity: c_double,
        fixed_strike: c_int,
    ) -> c_double;
    
    pub fn mco_lookback_put(
        ctx: *mut mco_context_t,
        spot: c_double,
        strike: c_double,
        rate: c_double,
        volatility: c_double,
        time_to_maturity: c_double,
        fixed_strike: c_int,
    ) -> c_double;
}
