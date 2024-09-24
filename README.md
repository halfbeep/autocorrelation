I use this tool for analysing the PAX time series from Kraken as a proxy for spot Gold prices. It is useful for detecting high-frequency changes (1 minute)

  $\hat\rho_k$​ = $\sum_{t = 1}^{n-k}  (X_t-\overline{X})(X_{t+k}-\overline{X})\over\sum_{t = 1}^{n}  (X_t-\overline{X})^2$

where:
-   $X_t$​ is the value of the return at time t
-   $\overline{X}$ is the mean of the returns
-    *k* is the lag (how many time steps back)
-   *n* is the total number of returns


Across a 3 hr observation window, 1 minute lag running for 36 hrs, autocorrelation ranged between -0.324 (high) and -0.608 (low); meaning the serial correlation of Gold never seems to be positive, albeit PAX is only a proxy
