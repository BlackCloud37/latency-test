use rand::RngCore;

pub fn random_buffer(buf: &mut [u8]) {
    rand::thread_rng().fill_bytes(buf);
}