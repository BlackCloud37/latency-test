use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_timestamp() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_micros() 
}

pub fn parse_ts(buf: [u8; 16]) -> u128 {
    let mut ts: u128 = 0;
    for n in buf {
        ts *= 10;
        ts += n as u128;
    }
    ts
}

pub fn format_ts(buf: &mut [u8; 16], ts: u128) {
    let mut ts = ts;
    for i in buf.iter_mut().rev() {
        *i = (ts % 10) as u8;
        ts /= 10;
    }
}

pub fn average(nums: &[u128]) -> f64 {
    nums.iter().sum::<u128>() as f64 / nums.len() as f64
}