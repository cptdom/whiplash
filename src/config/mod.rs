
use std::fs;
use std::error::Error;
use log::warn;
use serde_yaml;
use serde::{Deserialize, Serialize};

pub static DEFAULT_CONFIG_PATH: &str = "./config.yaml";
// const ATR_MAT: [&str; 3] = ["EMA", "RMA", "SMA"];
static DEFAULT_ATR_MAT: &str = "EMA";
const DEFAULT_ATR_CANDLES_PERCENT: f64 = 0.8;
const DEFAULT_ATR_THRESHOLD: f64 = 0.35;

// TODO: atr setting should be symbol-specific, not global
#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub atr_moving_average_type: String,
    pub atr_threshold: f64,
    pub atr_min_candles_percent: f64,
    pub min_vol_usdt: f64,
    pub symbols: Vec<String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Config, Box<dyn Error>> {
        let config_str = fs::read_to_string(path)?;
        let mut config: Config = serde_yaml::from_str(&config_str)?;
        if config.symbols.len() == 0 {
            Err("symbols are empty")?
        }
        if config.min_vol_usdt == 0. {
            Err("minimal volume value is empty")?
        }
        if config.atr_moving_average_type.to_uppercase() != DEFAULT_ATR_MAT {
            warn!("using default: {} as ATR moving average", DEFAULT_ATR_MAT);
            config.atr_moving_average_type = DEFAULT_ATR_MAT.to_string();
        }
        if config.atr_min_candles_percent <= 0. {
            warn!("using default: {:?} as atr min candles percent value", DEFAULT_ATR_CANDLES_PERCENT);
            config.atr_min_candles_percent = DEFAULT_ATR_CANDLES_PERCENT;
        }
        if config.atr_threshold <= 0. {
            warn!("using default: {:?} as atr threshold value", DEFAULT_ATR_CANDLES_PERCENT);
            config.atr_threshold = DEFAULT_ATR_THRESHOLD;
        }
        Ok(config)
    }
}
// TODO: impl config validation