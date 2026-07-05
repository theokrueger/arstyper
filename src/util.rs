use std::time::{SystemTime, UNIX_EPOCH};

pub fn timestamp_s() -> u64 {
    unsafe {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_unchecked()
            .as_secs()
    }
}
