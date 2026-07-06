use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ActionMap {
    bindings: BTreeMap<String, String>,
}

impl ActionMap {
    pub fn bind(&mut self, action: impl Into<String>, binding: impl Into<String>) {
        self.bindings.insert(action.into(), binding.into());
    }

    pub fn binding(&self, action: &str) -> Option<&str> {
        self.bindings.get(action).map(String::as_str)
    }
}
