pub enum Action {
    Code(String),
    Tool(String, String),
    Unknown,
}

impl Action {
    pub fn from_str(s: &str) -> Self {
        if s.starts_with("code(") {
            let code = s.trim_start_matches("code(")
                .trim_end_matches(")")
                .trim_matches('"')
                .to_string();
            Action::Code(code)
        } else {
            Action::Unknown
        }
    }
}
