use anyhow::Result;
use circular_buffer::CircularBuffer;
use chrono::{DateTime, Duration, Utc};
use super::event::Event;


#[derive(Debug, Clone, PartialEq)]
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
    pub fn from_kline_event(event: &Event) -> Result<Self> {
        let (kline_volume, close_price) = parse_kline_event(event)?;
        let ts = chrono::DateTime::from_timestamp_millis(event.E as i64)
            .ok_or_else(|| anyhow::anyhow!("invalid timestamp received"))?
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

fn parse_kline_event(event: &Event) -> Result<(f64, f64)> {
    let price_high: f64 = event.k.h.parse()?;
    let price_low: f64 = event.k.l.parse()?;
    let price_close: f64 = event.k.c.parse()?;
    let volume: f64 = event.k.v.parse()?;

    let event_size: f64 = (price_high + price_low) / 2. * volume;

    Ok((event_size, price_close))
}

pub fn calc_volume_delta(buffer: &SymbolBuffer, needed_seconds: i64) -> f64 {
    // info!("CVD: {:?}", buffer);
    // calculate the stop time
    let latest_timestamp = buffer.back().map(|node| node.ts).unwrap_or(Utc::now());
    let stop_time = latest_timestamp - Duration::seconds(needed_seconds);

    let mut total_volume_delta = 0.0;

    // go backwards
    for (iter, current_node) in buffer.iter().rev().enumerate() {
        // backup and peek ahead
        let current_node = current_node.clone();

        if let Some(previous_node) = buffer.iter().rev().nth(iter + 1) {
            if previous_node.ts <= stop_time || (previous_node.ts == latest_timestamp && iter != 0) {
                break;
            }

            if previous_node.confirmed && iter != 0 {
                total_volume_delta += current_node.value;
            } else {
                total_volume_delta += current_node.value - previous_node.value;
            }
        }
    }

    total_volume_delta
}


// TESTS
#[test]
fn test_calc_volume_data() {
    let latest_timestamp = Utc::now();

    // this node should be excluded
    let node0 = BufferNode {
        ts: latest_timestamp - Duration::milliseconds(2050),
        value: 0.1,
        confirmed: false,
        close_price:42.
    };
    let node1 = BufferNode {
        ts: latest_timestamp - Duration::milliseconds(1550),
        value: 1.0,
        confirmed: false,
        close_price:42.
    };
    let node2 = BufferNode {
        ts: latest_timestamp - Duration::milliseconds(1300),
        value: 2.0,
        confirmed: false,
        close_price:42.
    };
    let node3 = BufferNode {
        ts: latest_timestamp - Duration::milliseconds(1050),
        value: 3.0,
        confirmed: false,
        close_price:42.
    };
    let node4 = BufferNode {
        ts: latest_timestamp - Duration::milliseconds(800),
        value: 4.0,
        confirmed: true,
        close_price:42.
    };
    let node5 = BufferNode {
        ts: latest_timestamp - Duration::milliseconds(550),
        value: 1.0,
        confirmed: false,
        close_price:42.
    };
    let node6 = BufferNode {
        ts: latest_timestamp - Duration::milliseconds(300),
        value: 2.0,
        confirmed: false,
        close_price:42.
    };
    let node7 = BufferNode {
        ts: latest_timestamp - Duration::milliseconds(50),
        value: 3.0,
        confirmed: false,
        close_price:42.
    };

    let nodes = vec![node0, node1, node2, node3, node4, node5, node6, node7];
    let mut buffer = CircularBuffer::<244, BufferNode>::new();

    for node in nodes {
        buffer.push_back(node)
    }
    // copy buffer to make sure the final state is equal to the original one
    let buffer_backup = buffer.clone();
    // test for 2 seconds
    let volume_delta_over_2_seconds = calc_volume_delta(&mut buffer, 2);
    assert_eq!(volume_delta_over_2_seconds, 6.0);
    assert_eq!(buffer, buffer_backup);
    // test for 3 seconds - now the node at position 0 should be included
    let volume_delta_over_3_seconds = calc_volume_delta(&mut buffer, 3);
    assert_eq!(volume_delta_over_3_seconds, 6.9);
    assert_eq!(buffer, buffer_backup);
}
