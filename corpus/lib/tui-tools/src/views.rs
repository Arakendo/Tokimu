use crate::{
    split, Axis, LayoutConstraint, ProjectionArtifact, StyleRole, Surface, TextAlignment,
    TuiExtent, TuiInsets, TuiViewport, WrapMode, TUI_TOOLS_ARTIFACT_SCHEMA,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TranscriptLine {
    pub text: String,
    pub role: StyleRole,
}

impl TranscriptLine {
    pub fn new(text: impl Into<String>, role: StyleRole) -> Self {
        Self {
            text: text.into(),
            role,
        }
    }
}

pub fn render_transcript(
    title: &str,
    lines: &[TranscriptLine],
    viewport: TuiViewport,
    extent: TuiExtent,
) -> ProjectionArtifact {
    let mut surface = Surface::new(extent);
    let frame = extent.rect();
    surface.draw_frame(frame, StyleRole::Frame);
    let content = frame.inset(TuiInsets::all(1));
    if content.height == 0 {
        surface.extend_diagnostics([crate::TuiDiagnostic::EmptyViewport { viewport_rows: 0 }]);
        return ProjectionArtifact {
            schema: TUI_TOOLS_ARTIFACT_SCHEMA,
            producer: "tui-tools/transcript",
            extent,
            surface,
        };
    }
    surface.write_line(content, 0, title, TextAlignment::Start, StyleRole::Heading);
    let body = crate::TuiRect::new(
        content.x,
        content.y.saturating_add(1),
        content.width,
        content.height.saturating_sub(1),
    );
    let (start, end) = viewport.visible_rows();
    for (row, line) in lines[start.min(lines.len())..end.min(lines.len())]
        .iter()
        .take(usize::from(body.height))
        .enumerate()
    {
        surface.write_line(
            body,
            row as u16,
            &line.text,
            TextAlignment::Start,
            line.role,
        );
    }
    ProjectionArtifact {
        schema: TUI_TOOLS_ARTIFACT_SCHEMA,
        producer: "tui-tools/transcript",
        extent,
        surface,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusField {
    pub label: String,
    pub value: String,
    pub emphasized: bool,
}

impl StatusField {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            emphasized: false,
        }
    }

    pub fn emphasized(mut self) -> Self {
        self.emphasized = true;
        self
    }
}

/// Writes a bounded label/value row without making either field borrow the
/// other field's style. `value_column` is measured from the region's left
/// edge; values that do not fit remain explicitly clipped by `Surface`.
pub fn write_label_value_row(
    surface: &mut Surface,
    region: crate::TuiRect,
    row: u16,
    field: &StatusField,
    value_column: u16,
) {
    if region.is_empty() || row >= region.height {
        surface.extend_diagnostics([crate::TuiDiagnostic::EmptyRegion { region }]);
        return;
    }

    let value_column = value_column.min(region.width);
    let label_region = crate::TuiRect::new(region.x, region.y, value_column, region.height);
    surface.write_line(
        label_region,
        row,
        &field.label,
        TextAlignment::Start,
        StyleRole::Label,
    );

    let value_region = crate::TuiRect::new(
        region.x.saturating_add(value_column),
        region.y,
        region.width.saturating_sub(value_column),
        region.height,
    );
    surface.write_line(
        value_region,
        row,
        &field.value,
        TextAlignment::Start,
        if field.emphasized {
            StyleRole::Warning
        } else {
            StyleRole::Value
        },
    );
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusSection {
    pub title: String,
    pub fields: Vec<StatusField>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusDashboard {
    pub title: String,
    pub subtitle: String,
    pub sections: Vec<StatusSection>,
    pub footer: String,
}

pub fn render_status_dashboard(
    dashboard: &StatusDashboard,
    extent: TuiExtent,
) -> ProjectionArtifact {
    let mut surface = Surface::new(extent);
    let frame = extent.rect();
    surface.draw_frame(frame, StyleRole::Frame);
    let content = frame.inset(TuiInsets::all(1));
    let vertical = split(
        content,
        Axis::Vertical,
        &[
            LayoutConstraint::Fixed(2),
            LayoutConstraint::Remaining,
            LayoutConstraint::Fixed(1),
        ],
    );
    surface.extend_diagnostics(vertical.diagnostics);
    if vertical.regions.len() != 3 {
        return ProjectionArtifact {
            schema: TUI_TOOLS_ARTIFACT_SCHEMA,
            producer: "tui-tools/status-dashboard",
            extent,
            surface,
        };
    }

    let header = vertical.regions[0];
    surface.write_line(
        header,
        0,
        &dashboard.title,
        TextAlignment::Start,
        StyleRole::Heading,
    );
    surface.write_line(
        header,
        1,
        &dashboard.subtitle,
        TextAlignment::Start,
        StyleRole::Muted,
    );

    let body = vertical.regions[1];
    let columns = split(
        body,
        Axis::Horizontal,
        &[LayoutConstraint::Remaining, LayoutConstraint::Remaining],
    );
    surface.extend_diagnostics(columns.diagnostics);
    for (section_index, section) in dashboard.sections.iter().enumerate() {
        let Some(column) = columns.regions.get(section_index % 2).copied() else {
            break;
        };
        let block = column.inset(TuiInsets::symmetric(1, 0));
        let row_offset = (section_index / 2) as u16 * 7;
        if row_offset >= block.height {
            continue;
        }
        let section_region = crate::TuiRect::new(
            block.x,
            block.y.saturating_add(row_offset),
            block.width,
            block.height.saturating_sub(row_offset).min(7),
        );
        surface.write_line(
            section_region,
            0,
            &section.title,
            TextAlignment::Start,
            StyleRole::Accent,
        );
        for (row, field) in section.fields.iter().take(5).enumerate() {
            write_label_value_row(&mut surface, section_region, row as u16 + 1, field, 12);
        }
    }

    surface.write_text(
        vertical.regions[2],
        &dashboard.footer,
        WrapMode::Clip,
        StyleRole::Muted,
    );
    ProjectionArtifact {
        schema: TUI_TOOLS_ARTIFACT_SCHEMA,
        producer: "tui-tools/status-dashboard",
        extent,
        surface,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> StatusDashboard {
        StatusDashboard {
            title: "SYSTEM MONITOR".to_owned(),
            subtitle: "bounded synthetic evidence".to_owned(),
            sections: vec![StatusSection {
                title: "RUNTIME".to_owned(),
                fields: vec![StatusField::new("frame", "42")],
            }],
            footer: "Q quit | R reset".to_owned(),
        }
    }

    #[test]
    fn dashboard_is_repeatable() {
        let first = render_status_dashboard(&fixture(), TuiExtent::new(48, 14));
        let second = render_status_dashboard(&fixture(), TuiExtent::new(48, 14));
        assert_eq!(first, second);
        assert_eq!(first.schema, TUI_TOOLS_ARTIFACT_SCHEMA);
    }

    #[test]
    fn undersized_dashboard_remains_bounded_and_diagnostic() {
        let artifact = render_status_dashboard(&fixture(), TuiExtent::new(12, 3));
        assert_eq!(artifact.surface.cells().len(), 36);
        assert!(!artifact.surface.diagnostics().is_empty());
    }

    #[test]
    fn label_value_rows_keep_their_roles_and_value_column_when_bounded() {
        let mut surface = Surface::new(TuiExtent::new(14, 1));
        write_label_value_row(
            &mut surface,
            TuiExtent::new(14, 1).rect(),
            0,
            &StatusField::new("state", "ready").emphasized(),
            7,
        );

        assert_eq!(surface.to_plain_text(), "state  ready");
        assert_eq!(surface.cells()[0].role, StyleRole::Label);
        assert_eq!(surface.cells()[7].role, StyleRole::Warning);
    }
}
