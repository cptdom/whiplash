use std::collections::HashMap;
use std::error::Error;
use super::buffer::SymbolBuffer;
use chrono::{Duration, Timelike, Utc};

#[derive(Debug)]
pub struct ATRInputData {
    pub lows: Vec<f64>,
    pub highs: Vec<f64>,
    pub closes: Vec<f64>,
}


pub fn check_atr_condition(
    symbol: &String,
    buffer: &mut SymbolBuffer,
    seconds: usize,
    atr_threshold: f64,
    atr_min_candles_percent: f64
) -> Result<(bool, f64), Box<dyn Error>> {
    // Get ATR data
    let atr_input = get_atr_data(buffer, seconds)?;

    let mut seconds_to_fetch = seconds;
    let actual_atr_seconds = atr_input.closes.len();

    // Not enough candles to calculate accurate ATR
    if actual_atr_seconds < (seconds as f64 * atr_min_candles_percent).ceil() as usize {
        return Ok((false, 0.));
    }

    if seconds > actual_atr_seconds {
        seconds_to_fetch = actual_atr_seconds;
    }

    // Calculate ATR
    let calculated_atr = calculate_atr(&atr_input, seconds_to_fetch)?;
    let close_price = atr_input.closes[actual_atr_seconds - 1];

    if calculated_atr == 0.0 {
        return Ok((false, 0.));
    }

    let is_atr_limit_passed = (calculated_atr / close_price) > (atr_threshold / 100.0);

    // Log the result (assuming a logging system is available)
    log::debug!(
        "symbol: {}, atr: {}, close_price: {}, atr_min_percent: {}, atr_value: {}",
        symbol,
        calculated_atr,
        close_price,
        atr_threshold,
        is_atr_limit_passed
    );


    Ok((is_atr_limit_passed, calculated_atr))
}

fn calculate_atr(input: &ATRInputData, seconds: usize) -> Result<f64, Box<dyn Error>>{
    let atr_arr = atr_ema(&input.highs, &input.lows, &input.closes, seconds);
    let length  = atr_arr.len();
    if length != 0 {
        Ok(atr_arr[length-1])
    } else {
        Ok(0.)
    }

}

fn get_atr_data(buffer: &mut SymbolBuffer, seconds: usize) -> Result<ATRInputData, Box<dyn Error>> {
    if seconds > 60 {
        return Err("requested interval exceeds minute buffer length".into());
    }

    let saved_buffer_pointer = buffer.clone();

    let latest_timestamp = buffer.back().map(|node| node.ts).unwrap_or(Utc::now());
    let stop_time = latest_timestamp - Duration::seconds(seconds as i64);

    let mut key_order = Vec::new();
    let mut lows_map = HashMap::new();
    let mut highs_map = HashMap::new();
    let mut closes_map = HashMap::new();

    let mut iter = 0;
    while let Some(current_node) = buffer.back() {
        let current_node = current_node.clone();
        buffer.pop_back();

        let current_second = current_node.ts.second() as i32;

        if key_order.is_empty() || *key_order.last().unwrap() != current_second {
            key_order.push(current_second);
        }

        if closes_map.get(&current_second).is_none() {
            closes_map.insert(current_second, current_node.close_price);
        }

        let high_entry = highs_map.entry(current_second).or_insert(current_node.close_price);
        if current_node.close_price > *high_entry {
            *high_entry = current_node.close_price;
        }

        let low_entry = lows_map.entry(current_second).or_insert(current_node.close_price);
        if current_node.close_price < *low_entry {
            *low_entry = current_node.close_price;
        }

        if let Some(previous_node) = buffer.back() {
            if previous_node.ts <= stop_time || (previous_node.ts == latest_timestamp && iter != 0) {
                break;
            }
        }

        iter += 1;
    }

    // restore original position
    *buffer = saved_buffer_pointer;

    key_order.reverse();

    let mut lows = Vec::new();
    let mut highs = Vec::new();
    let mut closes = Vec::new();

    for key in key_order {
        lows.push(*lows_map.get(&key).unwrap());
        highs.push(*highs_map.get(&key).unwrap());
        closes.push(*closes_map.get(&key).unwrap());
    }

    let result = ATRInputData {
        lows,
        highs,
        closes,
    };

    Ok(result)
}

// atr calculation rewrites
// TODO: add SMA and RMA and make it configurable

// calculate atr using ema
pub fn atr_ema(in_high: &[f64], in_low: &[f64], in_close: &[f64], in_time_period: usize) -> Vec<f64> {
    let mut out_real = vec![0.0; in_close.len()];

    let in_time_period_f = in_time_period as f64;

    if in_time_period < 1 {
        return out_real;
    }

    if in_time_period <= 1 {
        return true_range(in_high, in_low, in_close);
    }

    let tr = true_range(in_high, in_low, in_close);
    let prev_atr_temp = calc_ema(&tr, in_time_period);
    let mut prev_atr = prev_atr_temp[in_time_period];
    out_real[in_time_period] = prev_atr;

    let mut out_idx = in_time_period + 1;
    let mut today = in_time_period + 1;

    while out_idx < in_close.len() {
        prev_atr = (prev_atr * (in_time_period_f - 1.0) + tr[today]) / in_time_period_f;
        out_real[out_idx] = prev_atr;
        today += 1;
        out_idx += 1;
    }

    out_real
}

fn calc_ema(in_real: &[f64], in_time_period: usize) -> Vec<f64> {
    let k = 2.0 / ((in_time_period + 1) as f64);
    ema(in_real, in_time_period, k)
}

fn ema(in_real: &[f64], in_time_period: usize, k1: f64) -> Vec<f64> {
    let mut out_real = vec![0.0; in_real.len()];
    let lookback_total = in_time_period - 1;
    let start_idx = lookback_total;
    let mut today = start_idx - lookback_total;

    // sma first
    let mut temp_real = 0.0;
    for _ in 0..in_time_period {
        temp_real += in_real[today];
        today += 1;
    }
    let mut prev_ma = temp_real / in_time_period as f64;

    // ema for the first element
    while today <= start_idx {
        prev_ma = ((in_real[today] - prev_ma) * k1) + prev_ma;
        today += 1;
    }
    out_real[start_idx] = prev_ma;
    let mut out_idx = start_idx + 1;

    // ema for the rest
    while today < in_real.len() {
        prev_ma = ((in_real[today] - prev_ma) * k1) + prev_ma;
        out_real[out_idx] = prev_ma;
        today += 1;
        out_idx += 1;
    }

    out_real
}


pub fn true_range(in_high: &[f64], in_low: &[f64], in_close: &[f64]) -> Vec<f64> {
    let len = in_close.len();
    let mut out_real = vec![0.0; len];

    let mut today = 1;
    while today < len {
        let temp_lt = in_low[today];
        let temp_ht = in_high[today];
        let temp_cy = in_close[today - 1];

        let greatest = f64::max(temp_ht - temp_lt, f64::max((temp_ht - temp_cy).abs(), (temp_lt - temp_cy).abs()));

        out_real[today] = greatest;
        today += 1;
    }

    out_real
}
