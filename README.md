## whiplash
Simple volatility monitor for crypto futures market

### what?
Whiplash's functionality is just a single feature of a bigger algorithmic trading project originally written in Go as a part of a stealth startup side business. Whiplash covers the part responsible for detecting volatility impulses on the crypto futures market.

The market is monitored using Binance's websocket symbol-specific stream. The goal is to detect volatility spikes that were a condition of a successful trading run in our strategy.

### but why?
I wanted a Rust exercise with a real use-case. This project at the moment is nothing but me trying to advance my understanding of Rust.

### usage
1. define a `config.yaml` file and follow the local example to understand how to populate the fields.
2. this project is using asynchronous Rust. The more symbols you monitor the less efficient whiplash will be. The bigger original used in production uses Go for its goroutines. Using threads here would be too heavyweight.
3. run with `whiplash --<path/to/config.yaml>`
4. marvel at the logs

#### TODO:
- config per symbol, not global
- config validation
- unit tests
- Dockerfile
- integration tests using a mock server and docker compose