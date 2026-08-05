use std::io;

use ratatui::{
    backend::{Backend, WindowSize},
    buffer::{Buffer, Cell},
    layout::{Position, Rect, Size},
};

/// Retained, bounded Ratatui target owned by the Tokimu consumer.
///
/// Ratatui submits only changed cells. Retaining them here lets Tokimu produce
/// pixels without routing terminal styling or glyph decisions through the
/// browser.
pub(crate) struct TokimuBackend {
    buffer: Buffer,
    cursor: Position,
    cursor_visible: bool,
}

impl TokimuBackend {
    pub(crate) fn new(width: u16, height: u16) -> Self {
        Self {
            buffer: Buffer::empty(Rect::new(0, 0, width, height)),
            cursor: Position::new(0, 0),
            cursor_visible: false,
        }
    }

    pub(crate) const fn buffer(&self) -> &Buffer {
        &self.buffer
    }
}

impl Backend for TokimuBackend {
    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        for (x, y, cell) in content {
            let destination = self.buffer.cell_mut(Position::new(x, y)).ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("Ratatui submitted cell ({x}, {y}) outside the Tokimu grid"),
                )
            })?;
            *destination = cell.clone();
        }
        Ok(())
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        self.cursor_visible = false;
        Ok(())
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        self.cursor_visible = true;
        Ok(())
    }

    fn get_cursor_position(&mut self) -> io::Result<Position> {
        Ok(self.cursor)
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> io::Result<()> {
        let position = position.into();
        if position.x >= self.buffer.area.width || position.y >= self.buffer.area.height {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "cursor position is outside the Tokimu grid",
            ));
        }
        self.cursor = position;
        Ok(())
    }

    fn clear(&mut self) -> io::Result<()> {
        self.buffer = Buffer::empty(self.buffer.area);
        Ok(())
    }

    fn size(&self) -> io::Result<Size> {
        Ok(Size::new(self.buffer.area.width, self.buffer.area.height))
    }

    fn window_size(&mut self) -> io::Result<WindowSize> {
        Ok(WindowSize {
            columns_rows: self.size()?,
            pixels: Size::new(0, 0),
        })
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn changed_cells_are_retained_without_erasing_neighbors() {
        let mut backend = TokimuBackend::new(2, 1);
        let mut first = Cell::default();
        first.set_char('A');
        backend
            .draw(std::iter::once((0, 0, &first)))
            .expect("first draw");

        let mut second = Cell::default();
        second.set_char('B');
        backend
            .draw(std::iter::once((1, 0, &second)))
            .expect("second draw");

        assert_eq!(backend.buffer().cell((0, 0)).unwrap().symbol(), "A");
        assert_eq!(backend.buffer().cell((1, 0)).unwrap().symbol(), "B");
    }
}
