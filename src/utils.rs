use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_timestamp() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros() 
}

pub fn parse_ts(buf: [u8; 16]) -> u128 {
    u128::from_be_bytes(buf)
}

pub fn format_ts(buf: &mut [u8; 16], ts: u128) {
    buf.clone_from_slice(&ts.to_be_bytes());
}

pub fn average(nums: &[u128]) -> f64 {
    nums.iter().sum::<u128>() as f64 / nums.len() as f64
}