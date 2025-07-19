use crate::agent::action::Action;

pub struct ParsedResponse {
    pub thought: String,
    pub action: Action,
}

pub fn parse_response(response: &str) -> ParsedResponse {
    let thought = extract("Thought:", response);
    let action_raw = extract("Action:", response);
    let action = Action::from_str(&action_raw);
    ParsedResponse { thought, action }
}

fn extract(prefix: &str, text: &str) -> String {
    text.lines()
        .find(|line| line.trim_start().starts_with(prefix))
        .map(|line| line.trim_start()[prefix.len()..].trim().to_string())
        .unwrap_or_default()
}
