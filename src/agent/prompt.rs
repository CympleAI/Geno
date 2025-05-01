pub fn build_prompt(user_input: &str) -> String {
    format!(
        "You are Geno.\nHuman: {}\nThought:",
        user_input
    )
}
