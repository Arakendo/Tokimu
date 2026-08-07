use crate::{TuiDiagnostic, TuiExtent, TuiRect};

pub const TUI_TOOLS_ARTIFACT_SCHEMA: u32 = 1;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StyleRole {
    #[default]
    Plain,
    Frame,
    Heading,
    Label,
    Value,
    Accent,
    Warning,
    Muted,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cell {
    pub symbol: char,
    pub role: StyleRole,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            symbol: ' ',
            role: StyleRole::Plain,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextAlignment {
    #[default]
    Start,
    Center,
    End,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum WrapMode {
    #[default]
    Clip,
    Word,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectionArtifact {
    pub schema: u32,
    pub producer: &'static str,
    pub extent: TuiExtent,
    pub surface: Surface,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Surface {
    extent: TuiExtent,
    cells: Vec<Cell>,
    diagnostics: Vec<TuiDiagnostic>,
}

impl Surface {
    pub fn new(extent: TuiExtent) -> Self {
        let cell_count = usize::from(extent.columns) * usize::from(extent.rows);
        Self {
            extent,
            cells: vec![Cell::default(); cell_count],
            diagnostics: Vec::new(),
        }
    }

    pub const fn extent(&self) -> TuiExtent {
        self.extent
    }

    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    pub fn diagnostics(&self) -> &[TuiDiagnostic] {
        &self.diagnostics
    }

    pub fn extend_diagnostics(&mut self, diagnostics: impl IntoIterator<Item = TuiDiagnostic>) {
        self.diagnostics.extend(diagnostics);
    }

    pub fn set(&mut self, x: u16, y: u16, symbol: char, role: StyleRole) -> bool {
        let Some(index) = self.index(x, y) else {
            return false;
        };
        self.cells[index] = Cell { symbol, role };
        true
    }

    pub fn draw_frame(&mut self, region: TuiRect, role: StyleRole) {
        if region.width < 2 || region.height < 2 {
            self.diagnostics.push(TuiDiagnostic::EmptyRegion { region });
            return;
        }
        let right = region.right() - 1;
        let bottom = region.bottom() - 1;
        self.set(region.x, region.y, '+', role);
        self.set(right, region.y, '+', role);
        self.set(region.x, bottom, '+', role);
        self.set(right, bottom, '+', role);
        for x in region.x + 1..right {
            self.set(x, region.y, '-', role);
            self.set(x, bottom, '-', role);
        }
        for y in region.y + 1..bottom {
            self.set(region.x, y, '|', role);
            self.set(right, y, '|', role);
        }
    }

    pub fn write_line(
        &mut self,
        region: TuiRect,
        row: u16,
        text: &str,
        alignment: TextAlignment,
        role: StyleRole,
    ) {
        if region.is_empty() || row >= region.height {
            self.diagnostics.push(TuiDiagnostic::EmptyRegion { region });
            return;
        }
        let characters: Vec<char> = text.chars().collect();
        let visible = characters.len().min(usize::from(region.width));
        let omitted = characters.len().saturating_sub(visible);
        let padding = usize::from(region.width).saturating_sub(visible);
        let start = match alignment {
            TextAlignment::Start => 0,
            TextAlignment::Center => padding / 2,
            TextAlignment::End => padding,
        };
        let y = region.y.saturating_add(row);
        for (offset, symbol) in characters.into_iter().take(visible).enumerate() {
            self.set(
                region.x.saturating_add((start + offset) as u16),
                y,
                symbol,
                role,
            );
        }
        if omitted > 0 {
            self.diagnostics.push(TuiDiagnostic::TextClipped {
                region,
                omitted_characters: omitted,
            });
        }
    }

    pub fn write_text(&mut self, region: TuiRect, text: &str, wrap: WrapMode, role: StyleRole) {
        if region.is_empty() {
            self.diagnostics.push(TuiDiagnostic::EmptyRegion { region });
            return;
        }
        let lines = match wrap {
            WrapMode::Clip => text.lines().map(str::to_owned).collect(),
            WrapMode::Word => wrap_words(text, usize::from(region.width)),
        };
        let omitted_lines = lines.len().saturating_sub(usize::from(region.height));
        for (row, line) in lines
            .into_iter()
            .take(usize::from(region.height))
            .enumerate()
        {
            self.write_line(region, row as u16, &line, TextAlignment::Start, role);
        }
        if omitted_lines > 0 {
            self.diagnostics.push(TuiDiagnostic::ContentTruncated {
                region,
                omitted_lines,
            });
        }
    }

    pub fn to_plain_text(&self) -> String {
        let mut output = String::new();
        for row in 0..self.extent.rows {
            let start = usize::from(row) * usize::from(self.extent.columns);
            let end = start + usize::from(self.extent.columns);
            let line: String = self.cells[start..end]
                .iter()
                .map(|cell| cell.symbol)
                .collect();
            output.push_str(line.trim_end());
            if row + 1 < self.extent.rows {
                output.push('\n');
            }
        }
        output
    }

    fn index(&self, x: u16, y: u16) -> Option<usize> {
        if x >= self.extent.columns || y >= self.extent.rows {
            return None;
        }
        Some(usize::from(y) * usize::from(self.extent.columns) + usize::from(x))
    }
}

fn wrap_words(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![String::new()];
    }
    let mut lines = Vec::new();
    for source_line in text.lines() {
        let mut current = String::new();
        for word in source_line.split_whitespace() {
            if word.chars().count() > width {
                if !current.is_empty() {
                    lines.push(std::mem::take(&mut current));
                }
                let characters: Vec<char> = word.chars().collect();
                for chunk in characters.chunks(width) {
                    lines.push(chunk.iter().collect());
                }
                continue;
            }
            let separator = usize::from(!current.is_empty());
            if current.chars().count() + separator + word.chars().count() > width {
                lines.push(std::mem::take(&mut current));
            }
            if !current.is_empty() {
                current.push(' ');
            }
            current.push_str(word);
        }
        if !current.is_empty() || source_line.is_empty() {
            lines.push(current);
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_is_clipped_to_its_region() {
        let mut surface = Surface::new(TuiExtent::new(8, 2));
        surface.write_line(
            TuiRect::new(2, 0, 3, 1),
            0,
            "abcdef",
            TextAlignment::Start,
            StyleRole::Value,
        );
        assert_eq!(surface.to_plain_text().lines().next(), Some("  abc"));
        assert!(matches!(
            surface.diagnostics().first(),
            Some(TuiDiagnostic::TextClipped {
                omitted_characters: 3,
                ..
            })
        ));
    }

    #[test]
    fn word_wrap_is_deterministic() {
        assert_eq!(
            wrap_words("alpha beta gamma", 7),
            vec!["alpha", "beta", "gamma"]
        );
    }

    #[test]
    fn styles_do_not_leak_between_cells() {
        let mut surface = Surface::new(TuiExtent::new(3, 1));
        surface.write_line(
            TuiRect::new(0, 0, 1, 1),
            0,
            "A",
            TextAlignment::Start,
            StyleRole::Accent,
        );
        assert_eq!(surface.cells()[0].role, StyleRole::Accent);
        assert_eq!(surface.cells()[1].role, StyleRole::Plain);
    }
}
