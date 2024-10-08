pub fn convert_from_bytes(bytes: u64, value: i32) -> f64 {
    bytes as f64 / f64::powf(1024., value as f64)
}
