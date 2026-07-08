use std::time::{SystemTime, UNIX_EPOCH};

pub fn timestamp_s() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
