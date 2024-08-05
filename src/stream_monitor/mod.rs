use anyhow::Result;
use async_std::task;
use buffer::BufferNode;
use circular_buffer::CircularBuffer;
use event::Event;
use futures_util::StreamExt;
use log::{error, info, debug};
use serde_json;
use std::sync::Arc;
use tokio::time::{interval, Duration};
use tokio::sync::Mutex;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::Message;

mod event;
mod buffer;
mod atr;

static FUTURES_URL: &str = "wss://fstream.binance.com/ws";
static STREAM_TYPE: &str = "kline_1m";
const ATR_CHECK_WINDOW_SECONDS: usize = 1;
const WARMUP_WINDOW_SECONDS: usize = 60;

pub struct SymbolData {
    pub symbol:  String,
    atr_threshold: f64,
    atr_min_candles_percent: f64,
    min_vol_usdt: f64,
    buffer: buffer::SymbolBuffer,
}

impl SymbolData {
    pub fn new(symbol: &str, atr_threshold: f64, atr_min_candles_percent: f64, min_vol_usdt: f64) -> Arc<Mutex<Self>> {
        let buffer = CircularBuffer::new();
        Arc::new(Mutex::new(
            SymbolData {
                symbol: symbol.to_string(),
                buffer: buffer,
                atr_threshold: atr_threshold,
                atr_min_candles_percent: atr_min_candles_percent,
                min_vol_usdt: min_vol_usdt,
            }
        ))
    }
}

pub async fn run(handler: Arc<Mutex<SymbolData>>) -> Result<()> {
    let url = {
        let handler = handler.lock().await;
        format!("{}/{}@{}", FUTURES_URL, handler.symbol.to_lowercase(), STREAM_TYPE)
    };

    info!("connecting to websocket at {}", url);

    // init connection now
    let (ws_stream, _) = connect_async(url).await?;
    debug!("connection successful");

    // split the stream into a receiver and a sender, we do not need the latter
    let (_, mut read) = ws_stream.split();

    // tokio magic
    let collection_clone = Arc::clone(&handler);

    // collection loop
    let collection_handle = tokio::spawn(async move {
        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    debug!("message received: {:?}", text);
                    match serde_json::from_str::<Event>(&text) {
                        Ok(parsed_message) => {

                            debug!("Received and parsed message: {:?}", parsed_message);
                            // append a new node to the buffer
                            match BufferNode::from_kline_event(&parsed_message) {
                                Ok(node) => {
                                    debug!("appending node: {:?}", node);
                                    let mut handler = collection_clone.lock().await;
                                    handler.buffer.push_back(node);
                                    // don't have to explicitly drop the lock because it goes out of the scope
                                    // and the lock is gone implicitly
                                }
                                Err(e) => {
                                    error!("failed to create BufferNode: {:?}", e);
                                }
                            }
                        }
                        Err(e) => {
                            error!("failed to parse message: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("error while reading from stream: {:?}", e)
                }
                _ => {}
            }
        }
    });

    let monitoring_clone = Arc::clone(&handler);
    let handler = handler.lock().await;
    let s = handler.symbol.clone();
    let at = handler.atr_threshold.clone();
    let am = handler.atr_min_candles_percent.clone();
    // explicitly drop the lock becasue it needs to be
    drop(handler);

    let monitoring_handle = tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(1));
        info!("allowing {:?} seconds to populate buffers...", WARMUP_WINDOW_SECONDS);
        task::sleep(Duration::from_secs(WARMUP_WINDOW_SECONDS as u64)).await;
        loop {
            interval.tick().await;
            let mut handler = monitoring_clone.lock().await;
            // calculate atr
            let atr_result = atr::check_atr_condition(
                &s,
                &mut handler.buffer, // Mutable reference to buffer
                ATR_CHECK_WINDOW_SECONDS,
                at,
                am
            );
            // get volume delta for the period
            let vol_usdt = buffer::calc_volume_delta(&mut handler.buffer, ATR_CHECK_WINDOW_SECONDS as i64);

            // Handle the ATR result as needed
            match atr_result {
                Ok((limit_passed, val)) => {
                   // TODO: this is to be changed based on the action we want to take
                   if limit_passed && vol_usdt >= handler.min_vol_usdt {
                    info!("SYMBOL {} READY FOR TRADE RUN, ATR: {:?}, VOLUME: {:?}", &s, val, vol_usdt)
                   } else {
                    info!("symbol {} idle, atr: {:?}, volume: {:?}", &s, val, vol_usdt)
                   }
                }
                Err(e) => {
                    error!("an error occurred while calculating atr for {}: {:?}", &s, e);
                }
            }
        }
    });

    let _ = tokio::try_join!(collection_handle, monitoring_handle);

    Ok(())
}

