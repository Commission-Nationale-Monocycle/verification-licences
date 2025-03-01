#[cfg(test)]
pub mod tests {
    use rand::random;
    use std::fs;
    use std::path::PathBuf;

    /// Create a new temp_dir with a random name.
    pub fn temp_dir() -> PathBuf {
        let buf = std::env::temp_dir().join(random::<u64>().to_string());
        fs::create_dir(&buf).unwrap();

        buf
    }
}
