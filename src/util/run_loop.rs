use chrono::NaiveDateTime;
use log::debug;
use std::collections::HashMap;
use std::env;
use std::io::Write;
use std::sync::{Arc, RwLock};
use tokio::time::{sleep, Duration};

#[path = "../calc/autocorrelation.rs"]
mod autocorrelation;
use autocorrelation::autocorrelation_of_returns;

#[path = "../util/rounding.rs"]
mod rounding;
use rounding::round_to_period;

#[path = "../data/kraken.rs"]
mod kraken;
use kraken::get_kraken_data;

// async data fetching function
async fn fetch_data_and_update_map(
    results_map: Arc<RwLock<HashMap<NaiveDateTime, Option<f64>>>>,
    time_period: &str, // Pass as a reference
) {
    match get_kraken_data(time_period).await {
        Ok(new_data) => {
            debug!("Fetched Kraken data: {:?}", new_data);

            {
                // Hold the lock only when updating the map, to avoid deadlocks
                let mut map = results_map.write().unwrap();
                debug!("Acquired write lock for updating map with new data");

                for (timestamp, price) in new_data {
                    let rounded_timestamp = round_to_period(timestamp, time_period);
                    map.entry(rounded_timestamp)
                        .and_modify(|e| *e = Some(price))
                        .or_insert(Some(price));
                }
                debug!("Updated results map with new data: {:?}", map);

                // Truncate results_map to the most recent NO_OF_PERIODS length
                let no_of_periods: usize = env::var("NO_OF_PERIODS")
                    .unwrap_or("100".to_string())
                    .parse()
                    .expect("NO_OF_PERIODS must be a valid integer");

                let mut timestamps: Vec<_> = map.keys().cloned().collect();
                timestamps.sort(); // Sorting in ascending order, oldest to newest

                if timestamps.len() > no_of_periods {
                    let excess_count = timestamps.len() - no_of_periods;
                    for timestamp in &timestamps[..excess_count] {
                        map.remove(timestamp);
                    }
                }

                debug!("Map size after trimming: {}", map.len());
            } // Write lock released here automatically when map goes out of scope
        }
        Err(e) => {
            println!("Failed to fetch Kraken data: {}", e);
        }
    }
}

// asynchronous for async sleep
pub async fn run_autocorrelation_loop(
    results_map: Arc<RwLock<HashMap<NaiveDateTime, Option<f64>>>>,
    time_period: String, // Keep ownership of time_period
) {
    let mut highest_ac = f64::MIN;
    let mut lowest_ac = f64::MAX;

    loop {
        // Use tokio's non-blocking sleep to avoid hanging the async runtime
        sleep(Duration::from_secs(30)).await;
        debug!("Woke up from sleep, starting new iteration...");

        // Fetch new data and update the map asynchronously
        fetch_data_and_update_map(Arc::clone(&results_map), &time_period).await; // Pass as a reference

        // Acquire read lock to compute autocorrelation
        let locked_map = results_map.read().unwrap();
        debug!("Acquired read lock to compute autocorrelation");

        // Collect and sort the results_map by timestamp
        let mut sorted_entries: Vec<_> = locked_map.iter().collect();
        sorted_entries.sort_by_key(|entry| entry.0); // Sort by the timestamp (the key)

        // No need to manually drop `locked_map`, it will be released after this scope

        // Find the most recent price (latest timestamp) from the sorted map
        if let Some((_latest_timestamp, latest_price)) = sorted_entries.last() {
            // Calculate the autocorrelation of returns
            let ac = autocorrelation_of_returns(&results_map);

            if let Some(ac_value) = ac {
                // Update the highest and lowest autocorrelation values
                if ac_value > highest_ac {
                    highest_ac = ac_value;
                }
                if ac_value < lowest_ac {
                    lowest_ac = ac_value;
                }

                // Print the latest price, autocorrelation, highest, and lowest on a single line
                if let Some(price) = latest_price {
                    print!(
                        "\rLast price: {}, Autocorrelation: {:.6}, Highest: {:.6}, Lowest: {:.6}",
                        price, ac_value, highest_ac, lowest_ac
                    );
                    std::io::stdout().flush().unwrap(); // Ensure the print is flushed to the console
                } else {
                    print!("\rNo valid price available for most recent timestamp.");
                    std::io::stdout().flush().unwrap();
                }
            } else {
                print!("\rNot enough data to compute autocorrelation.");
                std::io::stdout().flush().unwrap();
            }
        } else {
            print!("\rThe results_map is empty. No prices available.");
            std::io::stdout().flush().unwrap();
        }

        debug!("Completed iteration of the autocorrelation loop");
    }
}
