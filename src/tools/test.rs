#[cfg(test)]
pub mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::SystemTime;

    pub fn temp_dir() -> PathBuf {
        let buf = std::env::temp_dir().join(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_micros().to_string());
        fs::create_dir(&buf).unwrap();

        buf
    }
}