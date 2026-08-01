use ui_tools::{UiNodeId, UiTextInputOperation, UiTextInputState};

use crate::ui::{
    APPLY_ID, CANCEL_DELETE_ID, CONFIRM_DELETE_ID, DELETE_ID, FILTER_FIELD_ID, HOTSPOT_ID,
    NAME_FIELD_ID, NOTES_FIELD_ID, RESOURCE_ROW_BASE, REVERT_ID, VISIBILITY_ID,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ResourceKind {
    Mesh,
    Vector,
    Raster,
    Scene,
}

impl ResourceKind {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Mesh => "MESH",
            Self::Vector => "VECTOR",
            Self::Raster => "RASTER",
            Self::Scene => "SCENE",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SavedResource {
    name: String,
    notes: String,
    visible: bool,
    hotspot: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceDraft {
    pub id: u64,
    pub kind: ResourceKind,
    pub name: UiTextInputState,
    pub notes: UiTextInputState,
    pub visible: bool,
    pub hotspot: bool,
    saved: SavedResource,
}

impl ResourceDraft {
    fn new(id: u64, kind: ResourceKind, name: &str, notes: &str) -> Self {
        let saved = SavedResource {
            name: name.to_owned(),
            notes: notes.to_owned(),
            visible: true,
            hotspot: false,
        };
        Self {
            id,
            kind,
            name: UiTextInputState::new(saved.name.clone()),
            notes: UiTextInputState::new(saved.notes.clone()),
            visible: saved.visible,
            hotspot: saved.hotspot,
            saved,
        }
    }

    pub fn is_dirty(&self) -> bool {
        self.name.value() != self.saved.name
            || self.notes.value() != self.saved.notes
            || self.visible != self.saved.visible
            || self.hotspot != self.saved.hotspot
    }

    fn apply(&mut self) {
        self.saved = SavedResource {
            name: self.name.value().to_owned(),
            notes: self.notes.value().to_owned(),
            visible: self.visible,
            hotspot: self.hotspot,
        };
    }

    fn revert(&mut self) {
        self.name = UiTextInputState::new(self.saved.name.clone());
        self.notes = UiTextInputState::new(self.saved.notes.clone());
        self.visible = self.saved.visible;
        self.hotspot = self.saved.hotspot;
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResourceWorkbenchModel {
    pub filter: UiTextInputState,
    pub resources: Vec<ResourceDraft>,
    pub selected_id: u64,
    pub confirm_delete: bool,
    pub status: String,
}

impl Default for ResourceWorkbenchModel {
    fn default() -> Self {
        let resources = vec![
            ResourceDraft::new(1, ResourceKind::Scene, "Hangar", "Primary assembly scene"),
            ResourceDraft::new(2, ResourceKind::Mesh, "Hull", "Selectable outer shell"),
            ResourceDraft::new(3, ResourceKind::Mesh, "Rotor", "Animated mechanical group"),
            ResourceDraft::new(4, ResourceKind::Vector, "Safety Zone", "Inspection overlay"),
            ResourceDraft::new(
                5,
                ResourceKind::Raster,
                "Warning Grid",
                "Linear data texture",
            ),
            ResourceDraft::new(6, ResourceKind::Vector, "Telemetry", "Operator hotspot"),
        ];
        Self {
            filter: UiTextInputState::default(),
            resources,
            selected_id: 1,
            confirm_delete: false,
            status: "6 RESOURCES READY".to_owned(),
        }
    }
}

impl ResourceWorkbenchModel {
    pub fn selected(&self) -> &ResourceDraft {
        self.resources
            .iter()
            .find(|resource| resource.id == self.selected_id)
            .expect("selected resource must remain present")
    }

    fn selected_mut(&mut self) -> &mut ResourceDraft {
        let selected_id = self.selected_id;
        self.resources
            .iter_mut()
            .find(|resource| resource.id == selected_id)
            .expect("selected resource must remain present")
    }

    pub fn visible_resources(&self) -> Vec<&ResourceDraft> {
        let filter = self.filter.value().trim().to_ascii_lowercase();
        self.resources
            .iter()
            .filter(|resource| {
                filter.is_empty()
                    || resource.name.value().to_ascii_lowercase().contains(&filter)
                    || resource.kind.label().to_ascii_lowercase().contains(&filter)
            })
            .collect()
    }

    pub fn row_id(resource_id: u64) -> UiNodeId {
        UiNodeId(RESOURCE_ROW_BASE + resource_id)
    }

    pub fn apply_edit(&mut self, target: UiNodeId, operation: UiTextInputOperation) -> bool {
        match target {
            FILTER_FIELD_ID => self.filter.apply(operation),
            NAME_FIELD_ID => self.selected_mut().name.apply(operation),
            NOTES_FIELD_ID => self.selected_mut().notes.apply(operation),
            _ => return false,
        }
        self.status = "DRAFT CHANGED".to_owned();
        true
    }

    pub fn dismiss_modal(&mut self) -> bool {
        if !self.confirm_delete {
            return false;
        }
        self.confirm_delete = false;
        self.status = "DELETE CANCELLED".to_owned();
        true
    }

    pub fn activate(&mut self, target: UiNodeId) -> bool {
        if let Some(resource_id) = target.0.checked_sub(RESOURCE_ROW_BASE) {
            if self
                .resources
                .iter()
                .any(|resource| resource.id == resource_id)
            {
                self.selected_id = resource_id;
                self.confirm_delete = false;
                self.status = format!("SELECTED {}", self.selected().name.value().to_uppercase());
                return true;
            }
        }

        match target {
            VISIBILITY_ID => {
                let resource = self.selected_mut();
                resource.visible = !resource.visible;
                self.status = "VISIBILITY CHANGED".to_owned();
            }
            HOTSPOT_ID => {
                let resource = self.selected_mut();
                resource.hotspot = !resource.hotspot;
                self.status = "HOTSPOT CHANGED".to_owned();
            }
            APPLY_ID if self.selected().is_dirty() => {
                self.selected_mut().apply();
                self.status = "RESOURCE APPLIED".to_owned();
            }
            REVERT_ID if self.selected().is_dirty() => {
                self.selected_mut().revert();
                self.status = "DRAFT REVERTED".to_owned();
            }
            DELETE_ID if self.resources.len() > 1 => {
                self.confirm_delete = true;
                self.status = "CONFIRM DELETION".to_owned();
            }
            CANCEL_DELETE_ID => return self.dismiss_modal(),
            CONFIRM_DELETE_ID if self.confirm_delete && self.resources.len() > 1 => {
                let index = self
                    .resources
                    .iter()
                    .position(|resource| resource.id == self.selected_id)
                    .expect("selected resource must remain present");
                let deleted = self.resources.remove(index);
                let next = index.min(self.resources.len() - 1);
                self.selected_id = self.resources[next].id;
                self.confirm_delete = false;
                self.status = format!("DELETED {}", deleted.name.value().to_uppercase());
            }
            _ => return false,
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filtering_preserves_stable_resource_identity() {
        let model = ResourceWorkbenchModel {
            filter: UiTextInputState::new("mesh".to_owned()),
            ..ResourceWorkbenchModel::default()
        };
        let ids: Vec<_> = model
            .visible_resources()
            .into_iter()
            .map(|resource| ResourceWorkbenchModel::row_id(resource.id))
            .collect();
        assert_eq!(ids, vec![UiNodeId(102), UiNodeId(103)]);
    }

    #[test]
    fn dirty_commands_apply_and_revert_selected_state() {
        let mut model = ResourceWorkbenchModel::default();
        assert!(!model.activate(APPLY_ID));
        assert!(model.activate(HOTSPOT_ID));
        assert!(model.selected().is_dirty());
        assert!(model.activate(REVERT_ID));
        assert!(!model.selected().is_dirty());
        assert!(model.activate(VISIBILITY_ID));
        assert!(model.activate(APPLY_ID));
        assert!(!model.selected().is_dirty());
    }

    #[test]
    fn deletion_requires_confirmation_and_preserves_a_selection() {
        let mut model = ResourceWorkbenchModel::default();
        let original = model.resources.len();
        assert!(model.activate(DELETE_ID));
        assert_eq!(model.resources.len(), original);
        assert!(model.confirm_delete);
        assert!(model.activate(CONFIRM_DELETE_ID));
        assert_eq!(model.resources.len(), original - 1);
        assert!(model
            .resources
            .iter()
            .any(|item| item.id == model.selected_id));
    }
}
