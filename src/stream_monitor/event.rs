use serde::{Deserialize, Serialize};

// omitting all the properties we do not need to read at all
#[derive(Debug, Serialize, Deserialize)]
pub struct Kline {
    pub c: String,
    pub h: String,
    pub l: String,
    pub v: String,
    pub x: bool,
}
#[allow(non_snake_case)]
#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub E: u64,
    pub k: Kline,
}
