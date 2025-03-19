pub fn stringify<T: ToString>(e: T) -> String {
    format!("Error code: {}", e.to_string())
}
