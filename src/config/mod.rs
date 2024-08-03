
use std::fs;
use std::error::Error;
use serde_yaml;
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

pub static DEFAULT_CONFIG_PATH: &str = "./config.yaml";

// for later purposes
type SharedConfig = Arc<Mutex<Config>>;

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
    pub fn from_file(path: &str) -> Result<SharedConfig, Box<dyn Error>> {
        let config_str = fs::read_to_string(path)?;
        let config: Config = serde_yaml::from_str(&config_str)?;
        Ok(Arc::new(Mutex::new(config)))
    }
}
// TODO: impl config validation