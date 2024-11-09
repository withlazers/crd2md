pub fn to_anchor(s: &str) -> String {
    "#".to_string()
        + &s.to_lowercase()
            .chars()
            .filter_map(|c| {
                if c.is_ascii_whitespace() {
                    Some('-')
                } else if c.is_ascii_alphanumeric() {
                    Some(c)
                } else {
                    None
                }
            })
            .collect::<String>()
}
