use futures;
use std::env;
use std::error::Error;

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
    // Clone config data
    let symbols = config.symbols.clone();
    let atr_threshold = config.atr_threshold;
    let atr_min_candles_percent = config.atr_min_candles_percent;
    let min_vol_usdt = config.min_vol_usdt;
    info!("found configuration: {:?}", config);

    let mut handles = vec![];
    // for each configured symbol, run the collect & monitor loop
    for symbol in symbols {
        info!("init data for {}", symbol);
        // unwrap config here
        let handler = stream_monitor::SymbolData::new(
            symbol.as_str(),
            atr_threshold,
            atr_min_candles_percent,
            min_vol_usdt,
        );
        let outer_handle = tokio::spawn(async move {
            info!("starting monitoring loop for {}", symbol);
            if let Err(e) = stream_monitor::run(handler).await {
                error!("failed to start handler for {}: {:?}", symbol, e)
            }
        });
        handles.push(outer_handle);
    }

    // run until interrupted
    if let Err(e) = futures::future::try_join_all(handles).await {
        error!("failed to run the orchestra: {:?}", e);
    }
    tokio::signal::ctrl_c().await?;

    Ok(())
}

