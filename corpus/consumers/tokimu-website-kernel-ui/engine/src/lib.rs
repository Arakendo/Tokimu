//! Rust/WASM semantic adapter for the website kernel UI consumer.

use serde::Serialize;
use ui_resource_workbench::model::{ResourceDraft, ResourceWorkbenchModel};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct KernelUiSession {
    model: ResourceWorkbenchModel,
}

#[wasm_bindgen]
impl KernelUiSession {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            model: ResourceWorkbenchModel::default(),
        }
    }

    pub fn observation_json(&self) -> Result<String, JsValue> {
        self.snapshot()
    }

    pub fn set_filter(&mut self, value: &str) -> Result<String, JsValue> {
        self.model.set_filter(value);
        self.snapshot()
    }

    pub fn select_resource(&mut self, resource_id: u64) -> Result<String, JsValue> {
        self.model.select_resource(resource_id);
        self.snapshot()
    }

    pub fn set_name(&mut self, value: &str) -> Result<String, JsValue> {
        self.model.set_selected_name(value);
        self.snapshot()
    }

    pub fn set_notes(&mut self, value: &str) -> Result<String, JsValue> {
        self.model.set_selected_notes(value);
        self.snapshot()
    }

    pub fn toggle_visibility(&mut self) -> Result<String, JsValue> {
        self.model.toggle_selected_visibility();
        self.snapshot()
    }

    pub fn toggle_hotspot(&mut self) -> Result<String, JsValue> {
        self.model.toggle_selected_hotspot();
        self.snapshot()
    }

    pub fn apply(&mut self) -> Result<String, JsValue> {
        self.model.apply_selected();
        self.snapshot()
    }

    pub fn revert(&mut self) -> Result<String, JsValue> {
        self.model.revert_selected();
        self.snapshot()
    }

    pub fn request_delete(&mut self) -> Result<String, JsValue> {
        self.model.request_delete();
        self.snapshot()
    }

    pub fn cancel_delete(&mut self) -> Result<String, JsValue> {
        self.model.cancel_delete();
        self.snapshot()
    }

    pub fn confirm_delete(&mut self) -> Result<String, JsValue> {
        self.model.confirm_delete();
        self.snapshot()
    }

    pub fn dispose(&mut self) {
        self.model = ResourceWorkbenchModel::default();
    }
}

impl Default for KernelUiSession {
    fn default() -> Self {
        Self::new()
    }
}

impl KernelUiSession {
    fn snapshot(&self) -> Result<String, JsValue> {
        let selected = self.model.selected();
        let resources = self
            .model
            .visible_resources()
            .into_iter()
            .map(|resource| ResourceObservation::new(resource, self.model.selected_id))
            .collect::<Vec<_>>();
        serde_json::to_string(&WorkbenchObservation {
            schema: 1,
            status: &self.model.status,
            filter: self.model.filter.value(),
            selected_id: self.model.selected_id,
            total_count: self.model.resources.len(),
            visible_count: resources.len(),
            confirm_delete: self.model.confirm_delete,
            can_delete: self.model.resources.len() > 1,
            selected: ResourceObservation::new(selected, self.model.selected_id),
            resources,
        })
        .map_err(|error| JsValue::from_str(&format!("UI observation failed: {error}")))
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WorkbenchObservation<'a> {
    schema: u32,
    status: &'a str,
    filter: &'a str,
    selected_id: u64,
    total_count: usize,
    visible_count: usize,
    confirm_delete: bool,
    can_delete: bool,
    selected: ResourceObservation<'a>,
    resources: Vec<ResourceObservation<'a>>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ResourceObservation<'a> {
    id: u64,
    kind: &'static str,
    name: &'a str,
    notes: &'a str,
    visible: bool,
    hotspot: bool,
    dirty: bool,
    selected: bool,
}

impl<'a> ResourceObservation<'a> {
    fn new(resource: &'a ResourceDraft, selected_id: u64) -> Self {
        Self {
            id: resource.id,
            kind: resource.kind.label(),
            name: resource.name.value(),
            notes: resource.notes.value(),
            visible: resource.visible,
            hotspot: resource.hotspot,
            dirty: resource.is_dirty(),
            selected: resource.id == selected_id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn browser_observation_is_provider_neutral_and_model_owned() {
        let mut session = KernelUiSession::new();
        let filtered = session.set_filter("mesh").unwrap();
        assert!(filtered.contains(r#""visibleCount":2"#));
        let selected = session.select_resource(2).unwrap();
        assert!(selected.contains(r#""selectedId":2"#));
        assert!(selected.contains(r#""name":"Hull""#));
    }
}
