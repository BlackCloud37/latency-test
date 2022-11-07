use rand::RngCore;

pub fn average(nums: &[u128]) -> f64 {
    nums.iter().sum::<u128>() as f64 / nums.len() as f64
}

pub fn random_buffer(buf: &mut [u8]) {
    rand::thread_rng().fill_bytes(buf);
}