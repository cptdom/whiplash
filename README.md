## whiplash
Improvised volatility monitor for crypto futures market

### what?
Whiplash's functionality is just a single feature of a bigger algorithmic trading project originally written in Go. Whiplash covers the part responsible for detecting volatility impulses on the crypto futures market.

The market is monitored using Binance's websocket symbol-specific stream. The goal is to detect volatility spikes that were a condition of a successful trading run in a strategy. The fun part is that the data needed to calculate ATR and volume deltas is not provided directly, it needs to be calculated from the data returned in the events from Binance ws.

### but why?
I wanted a Rust exercise with a real use-case. This project at the moment is nothing but me trying to advance my understanding of Rust, while implementing something I first solved in Go some time ago.

**NOTE:** Right now, the project is just a feature rewrite and is probably not very Rust-y. I suggest not taking it as an example of any kind.

### usage
1. define a `config.yaml` file and follow the local example to understand how to populate the fields.
2. this project is using asynchronous Rust. The more symbols you monitor the less efficient whiplash will be. The bigger original used in production uses Go for its goroutines. Using threads here would be too heavyweight.
3. run with `whiplash --<path/to/config.yaml>`
4. marvel at the logs

#### TODO:
- config per symbol, not global
- CI GHA