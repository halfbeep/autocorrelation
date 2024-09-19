use anyhow::Result;
use std::collections::HashMap;
use chrono::{Duration, NaiveDateTime, Utc};
use dotenv::dotenv;
use log::debug;
use std::env;
use std::sync::{Arc, RwLock};
use tokio::task;

#[path = "./data/kraken.rs"]
mod kraken;
use kraken::get_kraken_data;

#[path = "./util/rounding.rs"]
mod rounding;
use rounding::round_to_period;

type ResultsMap = Arc<
    RwLock<
        HashMap<
            NaiveDateTime,
            (
                Option<f64>,
                Option<f64>,
            ),
        >,
    >,
>;

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
    // Validate that TIME_PERIOD is one of "second", "minute", "hour", or "day"
    if !["second", "minute", "hour", "day"].contains(&time_period.as_str()) {
        return Err(anyhow::anyhow!(
            "TIME_PERIOD must be one of: 'second', 'minute', 'hour', or 'day'."
        ));
    }

    debug!("No of periods {}, Time period {}", no_of_periods, time_period);
    

    // Convert `no_of_periods` to `i64`
    let no_of_periods_i64: i64 = no_of_periods.try_into().unwrap();

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

    // Initialize a results_map with a 5 price vector
    // (includes price 'VOLPrice' used for calculation)
    let results_map: ResultsMap = Arc::new(RwLock::new(HashMap::new()));

    // Fill in initial timestamps, creating
    // the placeholders for the volatility estimate
    {
        let mut map = results_map.write().unwrap();
        for _ in 0..no_of_periods {
            // Round the current timestamp to the specified time period
            let rounded_timestamp = round_to_period(current_timestamp, &time_period);

            // Insert the rounded timestamp into the map with default values
            map.insert(rounded_timestamp, (None, None));

            // Move to the previous time period
            current_timestamp = current_timestamp - time_duration;

            // Debug output to verify the timestamps
            debug!("{}", rounded_timestamp);
        }
    }

    let kraken_map = Arc::clone(&results_map);
    let kraken_time_period = time_period.clone();
    let kraken_task = task::spawn(async move {
        println!("Fetching Kraken data...");
        match get_kraken_data(&kraken_time_period).await {
            Ok(kraken_data) => {
                let mut map = kraken_map.write().unwrap();
                for (timestamp, average_price) in kraken_data {
                    let rounded_timestamp = round_to_period(timestamp, &kraken_time_period);
                    map.entry(rounded_timestamp)
                        .and_modify(|e| e.1 = Some(average_price))
                        .or_insert((None, Some(average_price)));
                    debug!("Time {}, Average price {}", rounded_timestamp, average_price);
                }
            } // Lock is Released here
            Err(e) => {
                println!("Failed to fetch Kraken data: {}", e);
            }
        }
    });

    // Await the kraken_task separately
    if let Err(e) = kraken_task.await {
        println!("Kraken task failed to complete: {:?}", e);
    }

    Ok(())
}
