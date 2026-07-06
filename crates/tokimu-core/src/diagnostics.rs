#[derive(Clone, Debug, Default)]
pub struct Diagnostics {
    startup_messages: Vec<String>,
}

impl Diagnostics {
    pub fn record(&mut self, message: impl Into<String>) {
        self.startup_messages.push(message.into());
    }

    pub fn startup_messages(&self) -> &[String] {
        &self.startup_messages
    }
}
