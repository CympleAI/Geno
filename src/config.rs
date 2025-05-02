pub struct AgentConfig {
    pub max_step: usize,
}

impl Default for AgentConfig {
    fn default() -> Self {
        Self {
            max_step: 10
        }
    }
}
