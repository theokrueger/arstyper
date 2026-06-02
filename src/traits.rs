use crate::ui::UiRequest;
use ratatui::{
    buffer::Buffer, crossterm::event::KeyEvent, layout::Rect, widgets::StatefulWidgetRef,
};
use std::{io, rc::Rc, sync::mpsc::SyncSender};

/// Renderable body screen for Arstyper
pub trait ArstyperScreen {
    /// Create a new empty version of this screen
    fn new(tx: SyncSender<UiRequest>) -> Self;

    /// Render this screen into the body
    fn render(&self, area: Rect, buf: &mut Buffer);

    /// Handle crossterm events for this screen
    fn handle_events(&mut self, key: KeyEvent);
}

/// Renderable Stateful Widget for Arstyper
pub trait ArstyperWidget: StatefulWidgetRef + Sized {
    /// Handle crossterm events for this widget
    fn handle_events(&mut self, key: KeyEvent, state: &mut Self::State);

    /// Create a new, empty version of this widget
    fn new(tx: SyncSender<UiRequest>) -> io::Result<Self>;
}

pub trait ArstyperWidgetState: Sized {
    fn new() -> io::Result<Self>;
}
