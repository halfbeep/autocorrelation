# Autocorrelation Tool

This Rust-based tool calculates the **autocorrelation** of time series data, specifically tailored for **PAX Gold (PAXG)** price data from the **Kraken** exchange, which is used as a proxy for spot gold prices. The tool is particularly useful for detecting high-frequency changes (e.g., 1-minute intervals) and analyzing the **realized momentum** of gold over short time periods.

## Overview

This tool fetches PAXG time series data from Kraken at 1-minute intervals and calculates the **autocorrelation** across a specified observation window. In finance, autocorrelation helps detect whether there is a pattern in the price movements over time. A high negative autocorrelation suggests a reversion to the mean, while a positive value could indicate momentum.

The tool runs over a **3-hour observation window**, calculating autocorrelations using a **1-minute lag** and can be run continuously for a longer period (e.g., **36 hours**).

### Key Findings (PAXG as a Proxy for Gold)

-   **Time Frame**: 1-minute high-frequency changes.
-   **Observation Window**: 3 hours, running for > 48 hours.
-   **Results**: The autocorrelation of PAXG ranged between **-0.324** and **-0.6488**. This suggests that the serial correlation of gold prices over short periods is consistently negative, indicating a tendency toward **mean reversion**. These results hold for PAXG, a solid proxy for spot gold.

## Features

-   **Fetches high-frequency (1-minute) PAXG price data** from the Kraken exchange.
-   **Calculates autocorrelation** of the PAXG time series for detecting momentum or mean-reversion trends.
-   Designed to **run continuously**, ideal for observing live autocorrelation over longer periods.
-   Outputs autocorrelation values as they update in real-time, including **highest and lowest observed autocorrelation**.

## Installation

Ensure you have **Rust** installed on your system. You can install Rust via [rustup](https://rustup.rs/).

1.  Clone the repository:
    
    bash
    
    Copy code
    
    `git clone https://github.com/halfbeep/autocorrelation  
    cd autocorrelation` 
    
2.  Set up the environment variables in a `.env` file for the number of periods and time period:
    
    makefile
    
    Copy code
    
    `NO_OF_PERIODS=180   # Set this to 180 periods (for 3 hours of 1-minute intervals)`
    `TIME_PERIOD=minute  # 1-minute intervals for autocorrelation` 
    
4.  Build the project:
    
    bash
    
    Copy code
    
    `cargo build` 
    
5.  Run the program:
    
    bash
    
    Copy code
    
    `cargo run` 
    

## Usage

The tool runs in a loop, fetching data from Kraken every 30 seconds and updating the autocorrelation calculation for a 3-hour window of 1-minute intervals. It prints the latest price, the current autocorrelation, and tracks the highest and lowest autocorrelations observed during the run.

Example output:

yaml

Copy code

`Last price: 2625.83, Autocorrelation: -0.578914, Highest: -0.324443, Lowest: -0.648800` 

## Configuration

You can configure the observation window and the period using the `.env` file or directly modifying environment variables.

-   **NO_OF_PERIODS**: Defines the length of the time series (e.g., 180 for 3 hours at 1-minute intervals).
-   **TIME_PERIOD**: Set the interval for each data point (`second`, `minute`, `hour`, `day`).

## Project Structure

-   **data/kraken.rs**: Contains functions for fetching PAXG price data from the Kraken exchange.
-   **util/rounding.rs**: Utilities for rounding timestamps to the specified time period.
-   **util/run_loop.rs**: Main loop responsible for fetching data and calculating autocorrelations.

## Autocorrelation Calculation

The autocorrelation is calculated based on the **returns** of the time series. The autocorrelation at lag kkk is given by:

${&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;}$<h1>$\hat\rho_k$​ = $\sum_{t = 1}^{n-k}  (X_t-\overline{X})(X_{t+k}-\overline{X})\over\sum_{t = 1}^{n}  (X_t-\overline{X})^2$</h1>

where:
- $\hat\rho_k$ is the autocorrelation of returns  $1&rarr;t$ using lag $k$
- $X_t$​ is the value of the return at time t
- $\overline{X}$ is the mean of the returns
- *k* is the lag (how many time steps back)
- *n* is the total number of returns

## Conclusion

This tool provides an efficient way to analyze the **realized momentum** of PAXG (and by extension, gold) over high-frequency time frames. The negative autocorrelation values indicate that short-term gold price movements tend toward mean reversion rather than momentum.

## License

This project is licensed under the MIT License.


