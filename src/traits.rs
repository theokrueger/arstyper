use crate::ui::{Styles, UiRequest};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::Rect,
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, Widget, Wrap},
};
use std::{cmp::min, rc::Rc, sync::mpsc::SyncSender, time::Instant};
/// Renderable body screen for Arstyper
pub trait ArstyperScreen {
    /// Create a new empty version of this screen
    fn new(s: Rc<Styles>, tx: SyncSender<UiRequest>) -> Self;
    fn render(&self, area: Rect, buf: &mut Buffer);
}
