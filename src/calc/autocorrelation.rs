use chrono::NaiveDateTime;
use log::debug;
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, RwLock}; // for datetime handling

/// Function to compute autocorrelation of the price returns given a lag
pub fn autocorrelation_of_returns(
    data: &Arc<RwLock<HashMap<NaiveDateTime, Option<f64>>>>, // Updated type
) -> Option<f64> {
    // Lock the data with a read lock
    let locked_data = data.read().unwrap();

    // Read the lag from the environment, or default to 12 if not set
    let lag: usize = env::var("AUTOC_LAG")
        .unwrap_or("15".to_string()) // Default to 12 periods if not set
        .parse()
        .expect("AUTOC_LAG must be a valid integer");

    // We will collect all the valid (non-None) prices from the data
    let prices: Vec<f64> = locked_data
        .values()
        .filter_map(|&price| price) // Collect only Some(f64) values
        .collect();

    // If there are fewer than 2 prices, we can't calculate returns, so return None
    if prices.len() < 2 {
        return None;
    }

    // Calculate returns (logarithmic or simple percentage change)
    let returns: Vec<f64> = prices
        .windows(2)
        .map(|pair| (pair[1] - pair[0]) / pair[0]) // simple return
        .collect();

    // Ensure there are enough returns to calculate the lagged autocorrelation
    if lag >= returns.len() {
        return None;
    }

    // Calculate mean of the returns
    let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;

    debug!("Mean {}", mean_return);

    debug!("{} Returns", returns.len());

    // Calculate mean of returnslet mean: f64 = returns.iter().sum::<f64>() / returns.len() as f64;

    // Calculate autocorrelation
    let numerator: f64 = returns
        .iter()
        .zip(returns.iter().skip(lag))
        .map(|(&r1, &r2)| (r1 - mean_return) * (r2 - mean_return))
        .sum();

    let denominator: f64 = returns.iter().map(|&r| (r - mean_return).powi(2)).sum();

    // Compute autocovariance and variance
    let autocovariance: f64 = returns
        .iter()
        .zip(&returns[lag..])
        .map(|(r1, r2)| (r1 - mean_return) * (r2 - mean_return))
        .sum::<f64>()
        / returns.len() as f64;

    let variance: f64 = returns
        .iter()
        .map(|r| (r - mean_return).powi(2))
        .sum::<f64>()
        / returns.len() as f64;

    debug!(
        "Covar's {} {}, Var's {} {}",
        numerator, autocovariance, denominator, variance
    );

    // Return the autocorrelation (which is autocovariance divided by variance)
    if variance != 0.0 {
        Some(autocovariance / variance)
    } else {
        None
    }
}
