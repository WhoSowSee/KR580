pub fn single(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
