use circular_buffer::CircularBuffer;
use chrono::{DateTime, Duration, Utc};
use super::event::Event;
use std::error::Error;


#[derive(Debug, Clone)]
pub struct BufferNode {
    pub value: f64,
    pub ts: DateTime<Utc>,
    pub confirmed: bool,
    pub close_price: f64
}

// we're collecting data for the last minute + some safe zone
const BUFFER_SIZE: usize = (60 + 1) * 4;

pub type SymbolBuffer = CircularBuffer<BUFFER_SIZE, BufferNode>;

impl BufferNode {
    pub fn from_kline_event(event: &Event) -> Result<Self, Box<dyn Error>> {
        let (kline_volume, close_price) = parse_kline_event(event)?;
        let ts = chrono::DateTime::from_timestamp_millis(event.E as i64)
            .ok_or("invalid timestamp")?
            .to_utc();

        let node: BufferNode = BufferNode {
            ts: ts,
            value: kline_volume,
            confirmed: event.k.x,
            close_price: close_price,
        };

        Ok(node)
    }
}

fn parse_kline_event(event: &Event) -> Result<(f64, f64), Box<dyn Error>> {
    let price_high: f64 = event.k.h.parse()?;
    let price_low: f64 = event.k.h.parse()?;
    let price_close: f64 = event.k.h.parse()?;
    let volume: f64 = event.k.h.parse()?;

    let event_size: f64 = (price_high + price_low) / 2. * volume;

    Ok((event_size, price_close))
}

pub fn calc_volume_delta(buffer: &mut SymbolBuffer, needed_seconds: i64) -> f64 {
    // calculate the stop time
    let latest_timestamp = buffer.back().map(|node| node.ts).unwrap_or(Utc::now());
    let stop_time = latest_timestamp - Duration::seconds(needed_seconds);

    let mut total_volume_delta = 0.0;
    let mut iter = 0;

    // store nodes temporarily during iteration
    let mut temp_nodes = Vec::new();

    // go backwards
    while let Some(current_node) = buffer.back() {
        // backup and peek ahead
        let current_node = current_node.clone();
        buffer.pop_back();
        temp_nodes.push(current_node.clone());

        if let Some(previous_node) = buffer.back() {
            if previous_node.ts <= stop_time || (previous_node.ts == latest_timestamp && iter != 0) {
                break;
            }

            if previous_node.confirmed && iter != 0 {
                total_volume_delta += current_node.value;
            } else {
                total_volume_delta += current_node.value - previous_node.value;
            }

            iter += 1;
        }
    }

    // Restore the buffer to its original state by re-pushing the popped nodes
    while let Some(node) = temp_nodes.pop() {
        buffer.push_front(node);
    }

    total_volume_delta
}

