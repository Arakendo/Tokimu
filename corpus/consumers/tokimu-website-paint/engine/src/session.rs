use crate::{
    export_png, sample_color, DocumentConfig, DocumentError, DocumentObservation, EditObservation,
    EditableRasterDocument, ExportConfig, ExportError, HistoryConfig, HistoryError,
    HistoryObservation, LosslessExport, PaintCommand, PaintWorkspace, PixelPoint, Rgba8,
};
use raster_image_corpus::DecodedImage;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct PaintSessionConfig {
    pub document: DocumentConfig,
    pub history: HistoryConfig,
    pub export: ExportConfig,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionObservation {
    pub schema: u32,
    pub active: bool,
    pub document: Option<DocumentObservation>,
    pub history: Option<HistoryObservation>,
}

/// A bounded copy of authoritative RGBA pixels for a presentation adapter.
/// The caller receives bytes, not a browser Canvas or renderer texture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentPreview {
    pub width: u32,
    pub height: u32,
    pub row_stride: usize,
    pub pixels: Vec<u8>,
    pub pixel_fingerprint: String,
}

#[derive(Clone, Debug)]
pub struct PaintSession {
    config: PaintSessionConfig,
    initial_document: Option<EditableRasterDocument>,
    workspace: Option<PaintWorkspace>,
}

impl PaintSession {
    pub fn new_blank(
        width: u32,
        height: u32,
        color: Rgba8,
        config: PaintSessionConfig,
    ) -> Result<Self, SessionError> {
        let document = EditableRasterDocument::blank(width, height, color, config.document)?;
        Ok(Self::with_document(document, config))
    }

    pub fn open_decoded(
        source: &DecodedImage,
        config: PaintSessionConfig,
    ) -> Result<Self, SessionError> {
        let document = EditableRasterDocument::from_decoded(source, config.document)?;
        Ok(Self::with_document(document, config))
    }

    pub fn observation(&self) -> SessionObservation {
        let Some(workspace) = &self.workspace else {
            return SessionObservation {
                schema: 1,
                active: false,
                document: None,
                history: None,
            };
        };
        SessionObservation {
            schema: 1,
            active: true,
            document: Some(workspace.document().observation()),
            history: Some(workspace.history_observation()),
        }
    }

    pub fn apply(&mut self, command: &PaintCommand) -> Result<EditObservation, SessionError> {
        Ok(self.workspace_mut()?.apply(command)?)
    }

    pub fn undo(&mut self) -> Result<bool, SessionError> {
        Ok(self.workspace_mut()?.undo())
    }

    pub fn redo(&mut self) -> Result<bool, SessionError> {
        Ok(self.workspace_mut()?.redo())
    }

    pub fn sample(&self, point: PixelPoint) -> Result<Rgba8, SessionError> {
        Ok(sample_color(self.workspace()?.document(), point)?)
    }

    pub fn preview(&self) -> Result<DocumentPreview, SessionError> {
        let document = self.workspace()?.document();
        let observation = document.observation();
        Ok(DocumentPreview {
            width: document.width(),
            height: document.height(),
            row_stride: document.row_stride(),
            pixels: document.pixels().to_vec(),
            pixel_fingerprint: observation.pixel_fingerprint,
        })
    }

    pub fn export_png(&self) -> Result<LosslessExport, SessionError> {
        Ok(export_png(
            self.workspace()?.document(),
            self.config.export,
        )?)
    }

    pub fn reset_history(&mut self) -> Result<(), SessionError> {
        self.workspace_mut()?.reset_history();
        Ok(())
    }

    /// Restores the document supplied at session creation and releases history.
    /// This is document state restoration, never a Canvas readback.
    pub fn reset(&mut self) -> Result<SessionObservation, SessionError> {
        let document = self
            .initial_document
            .clone()
            .ok_or(SessionError::Disposed)?;
        self.workspace = Some(PaintWorkspace::new(document, self.config.history));
        Ok(self.observation())
    }

    pub fn dispose(&mut self) {
        self.initial_document = None;
        self.workspace = None;
    }

    fn with_document(document: EditableRasterDocument, config: PaintSessionConfig) -> Self {
        Self {
            initial_document: Some(document.clone()),
            workspace: Some(PaintWorkspace::new(document, config.history)),
            config,
        }
    }

    fn workspace(&self) -> Result<&PaintWorkspace, SessionError> {
        self.workspace.as_ref().ok_or(SessionError::Disposed)
    }

    fn workspace_mut(&mut self) -> Result<&mut PaintWorkspace, SessionError> {
        self.workspace.as_mut().ok_or(SessionError::Disposed)
    }
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum SessionError {
    #[error("Paint session has been disposed")]
    Disposed,
    #[error(transparent)]
    Document(#[from] DocumentError),
    #[error(transparent)]
    History(#[from] HistoryError),
    #[error(transparent)]
    Export(#[from] ExportError),
    #[error(transparent)]
    Command(#[from] crate::CommandError),
}

#[cfg(test)]
mod tests {
    use super::{PaintSession, PaintSessionConfig, SessionError};
    use crate::{PaintCommand, PixelPoint, Rgba8};

    const BLACK: Rgba8 = Rgba8 {
        red: 0,
        green: 0,
        blue: 0,
        alpha: 255,
    };
    const WHITE: Rgba8 = Rgba8 {
        red: 255,
        green: 255,
        blue: 255,
        alpha: 255,
    };

    #[test]
    fn session_coordinates_commands_preview_history_and_export() {
        let mut session =
            PaintSession::new_blank(2, 1, BLACK, PaintSessionConfig::default()).unwrap();
        let initial = session.observation();
        session
            .apply(&PaintCommand::PencilStroke {
                points: vec![PixelPoint { x: 0, y: 0 }],
                color: WHITE,
            })
            .unwrap();

        assert!(session.observation().document.unwrap().dirty);
        assert_eq!(session.preview().unwrap().pixels[..4], [255, 255, 255, 255]);
        assert!(session.undo().unwrap());
        assert_eq!(
            session.observation().document.unwrap().pixel_fingerprint,
            initial.document.unwrap().pixel_fingerprint
        );
        assert!(session.redo().unwrap());
        assert_eq!(
            session.export_png().unwrap().observation.format,
            "png-rgba8"
        );
    }

    #[test]
    fn preview_bytes_are_an_exact_authoritative_document_snapshot() {
        let mut session =
            PaintSession::new_blank(2, 1, BLACK, PaintSessionConfig::default()).unwrap();
        session
            .apply(&PaintCommand::PencilStroke {
                points: vec![PixelPoint { x: 0, y: 0 }],
                color: WHITE,
            })
            .unwrap();

        let preview = session.preview().unwrap();
        let observation = session.observation();
        assert_eq!(preview.width, 2);
        assert_eq!(preview.height, 1);
        assert_eq!(preview.pixels, vec![255, 255, 255, 255, 0, 0, 0, 255]);
        assert_eq!(
            preview.pixel_fingerprint,
            observation.document.unwrap().pixel_fingerprint
        );
    }

    #[test]
    fn disposed_sessions_reject_all_ownership_operations_predictably() {
        let mut session =
            PaintSession::new_blank(1, 1, BLACK, PaintSessionConfig::default()).unwrap();
        session.dispose();

        assert!(!session.observation().active);
        assert_eq!(session.preview(), Err(SessionError::Disposed));
        assert_eq!(session.undo(), Err(SessionError::Disposed));
        assert_eq!(session.reset(), Err(SessionError::Disposed));
    }

    #[test]
    fn reset_restores_the_initial_document_and_releases_history() {
        let mut session =
            PaintSession::new_blank(2, 1, BLACK, PaintSessionConfig::default()).unwrap();
        let initial = session.observation();
        session
            .apply(&PaintCommand::PencilStroke {
                points: vec![PixelPoint { x: 0, y: 0 }],
                color: WHITE,
            })
            .unwrap();

        let reset = session.reset().unwrap();
        assert_eq!(reset.document.as_ref(), initial.document.as_ref());
        let history = reset.history.as_ref().unwrap();
        assert_eq!(history.undo_depth, 0);
        assert_eq!(history.redo_depth, 0);
    }
}
