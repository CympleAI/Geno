pub enum Action {
    GenerateCode(String),
    Unknown,
}

impl Action {
    pub fn from_str(s: &str) -> Self {
        if s.starts_with("generate_code(") {
            let code = s.trim_start_matches("generate_code(")
                .trim_end_matches(")")
                .trim_matches('"')
                .to_string();
            Action::GenerateCode(code)
        } else {
            Action::Unknown
        }
    }
}

pub fn execute_action(action: Action) -> String {
    match action {
        Action::GenerateCode(code) => format!("模拟生成的代码:\n{}", code),
        Action::Unknown => "未知操作".to_string(),
    }
}
