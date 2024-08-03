use std::env;
use std::error::Error;
use std::sync::Arc;

use log::{error, info, warn};

pub mod config;
pub mod stream_monitor;
pub mod util;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    util::init_logger();
    info!("initializing whiplash");
    // get the config path from args
    let args: Vec<String> = env::args().collect();
    let mut config_path = config::DEFAULT_CONFIG_PATH;
    if args.len() != 2 {
        warn!("config path not specified, using default {:?}", config::DEFAULT_CONFIG_PATH);
    } else {
        config_path = &args[1];
    }
    // load the config using the path
    let config = config::Config::from_file(config_path)?;
    let config_clone = Arc::clone(&config);
    let config = config.lock().unwrap();
    info!("found configuration: {:?}", config);


    // for each configured symbol, run the collect & monitor loop
    for symbol in config.symbols.clone() {
        // unwrap config here
        let c = config_clone.lock().unwrap();
        let handler = stream_monitor::SymbolData::new(
            symbol.as_str(),
            c.atr_threshold,
            c.atr_min_candles_percent,
            c.min_vol_usdt,
        );
        tokio::spawn(async move {
            if let Err(e) = stream_monitor::run(handler).await {
                error!("failed to start handler for {}: {:?}", symbol, e)
            }
        });
    }

    // run until interrupted
    tokio::signal::ctrl_c().await?;

    Ok(())
}

