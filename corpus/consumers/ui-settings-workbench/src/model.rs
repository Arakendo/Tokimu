use ui_tools::{UiNodeId, UiTextInputOperation, UiTextInputState};

use crate::ui::{
    APPLY_ID, AUTHOR_FIELD_ID, DIAGNOSTICS_ID, PROJECT_FIELD_ID, QUALITY_ID, RESET_ID,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderQuality {
    Balanced,
    Detailed,
}

impl RenderQuality {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Balanced => "BALANCED",
            Self::Detailed => "DETAILED",
        }
    }

    const fn toggled(self) -> Self {
        match self {
            Self::Balanced => Self::Detailed,
            Self::Detailed => Self::Balanced,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SavedSettings {
    project_name: String,
    author: String,
    diagnostics: bool,
    quality: RenderQuality,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SettingsModel {
    pub project_name: UiTextInputState,
    pub author: UiTextInputState,
    pub diagnostics: bool,
    pub quality: RenderQuality,
    saved: SavedSettings,
    pub status: String,
}

impl Default for SettingsModel {
    fn default() -> Self {
        let saved = SavedSettings {
            project_name: "Tokimu Workshop".to_owned(),
            author: "Engine Team".to_owned(),
            diagnostics: true,
            quality: RenderQuality::Balanced,
        };
        Self {
            project_name: UiTextInputState::new(saved.project_name.clone()),
            author: UiTextInputState::new(saved.author.clone()),
            diagnostics: saved.diagnostics,
            quality: saved.quality,
            saved,
            status: "SETTINGS ARE IN SYNC".to_owned(),
        }
    }
}

impl SettingsModel {
    pub fn is_dirty(&self) -> bool {
        self.project_name.value() != self.saved.project_name
            || self.author.value() != self.saved.author
            || self.diagnostics != self.saved.diagnostics
            || self.quality != self.saved.quality
    }

    pub fn apply_edit(&mut self, target: UiNodeId, operation: UiTextInputOperation) -> bool {
        let field = match target {
            PROJECT_FIELD_ID => &mut self.project_name,
            AUTHOR_FIELD_ID => &mut self.author,
            _ => return false,
        };
        field.apply(operation);
        self.status = "DRAFT CHANGED".to_owned();
        true
    }

    pub fn activate(&mut self, target: UiNodeId) -> bool {
        match target {
            DIAGNOSTICS_ID => {
                self.diagnostics = !self.diagnostics;
                self.status = "DIAGNOSTICS CHANGED".to_owned();
            }
            QUALITY_ID => {
                self.quality = self.quality.toggled();
                self.status = "QUALITY CHANGED".to_owned();
            }
            APPLY_ID if self.is_dirty() => {
                self.saved = SavedSettings {
                    project_name: self.project_name.value().to_owned(),
                    author: self.author.value().to_owned(),
                    diagnostics: self.diagnostics,
                    quality: self.quality,
                };
                self.status = "SETTINGS APPLIED".to_owned();
            }
            RESET_ID if self.is_dirty() => {
                self.project_name = UiTextInputState::new(self.saved.project_name.clone());
                self.author = UiTextInputState::new(self.saved.author.clone());
                self.diagnostics = self.saved.diagnostics;
                self.quality = self.saved.quality;
                self.status = "DRAFT RESET".to_owned();
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
    fn edits_are_owned_by_the_targeted_application_field() {
        let mut model = SettingsModel::default();
        let author_before = model.author.value().to_owned();

        assert!(model.apply_edit(PROJECT_FIELD_ID, UiTextInputOperation::Insert('!')));
        assert!(model.project_name.value().ends_with('!'));
        assert_eq!(model.author.value(), author_before);
        assert!(model.is_dirty());
    }

    #[test]
    fn apply_and_reset_follow_application_dirty_state() {
        let mut model = SettingsModel::default();
        assert!(!model.activate(APPLY_ID));
        assert!(!model.activate(RESET_ID));

        assert!(model.activate(DIAGNOSTICS_ID));
        assert!(model.is_dirty());
        assert!(model.activate(RESET_ID));
        assert!(!model.is_dirty());

        assert!(model.activate(QUALITY_ID));
        assert!(model.activate(APPLY_ID));
        assert!(!model.is_dirty());
        assert_eq!(model.status, "SETTINGS APPLIED");
    }
}
