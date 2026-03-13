use crate::ui::{Styles, UiRequest};
use ratatui::{buffer::Buffer, crossterm::event::KeyEvent, layout::Rect};
use std::{rc::Rc, sync::mpsc::SyncSender};

/// Renderable body screen for Arstyper
pub trait ArstyperScreen {
    /// Create a new empty version of this screen
    fn new(s: Rc<Styles>, tx: SyncSender<UiRequest>) -> Self;

    /// Render this screen into the body
    fn render(&self, area: Rect, buf: &mut Buffer);

    /// Handle crossterm events for this screen
    fn handle_events(&mut self, key: KeyEvent);
}
