use anyhow::Result;
use chrono::{Duration, NaiveDateTime, Utc};
use dotenv::dotenv;
use log::debug;
use std::collections::HashMap;
use std::env;
use std::sync::{Arc, RwLock};

#[path = "./util/rounding.rs"]
mod rounding;
use rounding::round_to_period;

#[path = "./util/run_loop.rs"]
mod run_loop;
use run_loop::run_autocorrelation_loop;

type ResultsMap = Arc<RwLock<HashMap<NaiveDateTime, Option<f64>>>>;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize the logger once at the start of the program
    if env_logger::try_init().is_err() {
        eprintln!("Logger was already initialized");
    }
    dotenv().ok();

    // Load the number of periods from the .env file
    let no_of_periods: usize = env::var("NO_OF_PERIODS")
        .unwrap_or("100".to_string()) // Default to 100 periods if not set
        .parse()
        .expect("NO_OF_PERIODS must be a valid integer");

    // Check that NO_OF_PERIODS is in a reasonable range
    if no_of_periods == 0 || no_of_periods >= 741 {
        return Err(anyhow::anyhow!(
            "NO_OF_PERIODS must be greater than 0 and less than 741"
        ));
    }

    // Default to hour if period is absent
    let time_period = env::var("TIME_PERIOD").unwrap_or("hour".to_string());
    if !["second", "minute", "hour", "day"].contains(&time_period.as_str()) {
        return Err(anyhow::anyhow!(
            "TIME_PERIOD must be one of: 'second', 'minute', 'hour', or 'day'."
        ));
    }

    debug!(
        "No of periods {}, Time period {}",
        no_of_periods, time_period
    );

    // Determine the duration based on TIME_PERIOD
    let time_duration = match time_period.as_str() {
        "second" => Duration::seconds(1),
        "minute" => Duration::minutes(1),
        "hour" => Duration::hours(1),
        "day" => Duration::days(1),
        _ => Duration::hours(1), // Default to 'hour' if the provided value is invalid
    };

    // Initialize the starting timestamp (now - time_period)
    let mut current_timestamp = Utc::now().naive_utc();

    // Initialize a results_map with an Option<f64> placeholder for prices
    let results_map: ResultsMap = Arc::new(RwLock::new(HashMap::new()));

    // Fill in initial timestamps with None values
    {
        let mut map = results_map.write().unwrap();
        for _ in 0..no_of_periods {
            let rounded_timestamp = round_to_period(current_timestamp, &time_period);
            map.insert(rounded_timestamp, None);
            current_timestamp = current_timestamp - time_duration;
            debug!("{}", rounded_timestamp);
        }
    }

    // Run the Kraken task concurrently
    // let kraken_map = Arc::clone(&results_map);
    // let kraken_time_period = time_period.clone();

    // Spawn the autocorrelation loop
    let autocorrelation_task = tokio::spawn(run_autocorrelation_loop(
        Arc::clone(&results_map),
        time_period,
    ));

    // Await the autocorrelation task
    if let Err(e) = autocorrelation_task.await {
        println!("Autocorrelation loop task failed to complete: {:?}", e);
    }

    Ok(())
}
