use crate::ui::{Styles, UiRequest};
use ratatui::{buffer::Buffer, layout::Rect};
use std::{rc::Rc, sync::mpsc::SyncSender};

/// Renderable body screen for Arstyper
pub trait ArstyperScreen {
    /// Create a new empty version of this screen
    fn new(s: Rc<Styles>, tx: SyncSender<UiRequest>) -> Self;
    fn render(&self, area: Rect, buf: &mut Buffer);
}
