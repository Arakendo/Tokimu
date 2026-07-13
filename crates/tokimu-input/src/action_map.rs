use std::collections::BTreeMap;

use crate::{ControllerButton, ControllerId, KeyCode, MouseButton};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum InputBinding {
    Key(KeyCode),
    MouseButton(MouseButton),
    ControllerButton {
        controller: ControllerId,
        button: ControllerButton,
    },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ActionMap {
    bindings: BTreeMap<String, Vec<InputBinding>>,
}

impl ActionMap {
    pub fn bind(&mut self, action: impl Into<String>, binding: InputBinding) {
        self.bindings.entry(action.into()).or_default().push(binding);
    }

    pub fn bindings(&self, action: &str) -> &[InputBinding] {
        self.bindings.get(action).map(Vec::as_slice).unwrap_or(&[])
    }

    pub fn is_bound(&self, action: &str, binding: InputBinding) -> bool {
        self.bindings(action).contains(&binding)
    }

    pub fn clear(&mut self, action: &str) {
        self.bindings.remove(action);
    }
}

#[cfg(test)]
mod tests {
    use super::{ActionMap, InputBinding};
    use crate::{ControllerButton, KeyCode};

    #[test]
    fn action_map_can_store_multiple_bindings() {
        let mut map = ActionMap::default();

        map.bind("jump", InputBinding::Key(KeyCode::Space));
        map.bind(
            "jump",
            InputBinding::ControllerButton {
                controller: 0,
                button: ControllerButton::South,
            },
        );

        assert!(map.is_bound("jump", InputBinding::Key(KeyCode::Space)));
        assert!(map.is_bound(
            "jump",
            InputBinding::ControllerButton {
                controller: 0,
                button: ControllerButton::South,
            }
        ));
    }
}
