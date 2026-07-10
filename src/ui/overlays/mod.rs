pub mod language_select;
pub mod menu;

use crate::traits::{ArstyperOverlay, ArstyperWidget, ArstyperWidgetState};
use crate::ui::UiRequest;
use ratatui::{buffer::Buffer, crossterm::event::KeyEvent, layout::Rect};
use std::{io, sync::mpsc::SyncSender};

pub trait OverlayLayout: ArstyperWidget
where
    Self::State: ArstyperWidgetState,
{
    fn overlay_area(area: Rect) -> Rect;
}

pub struct OverlayWrapper<W: OverlayLayout>
where
    W::State: ArstyperWidgetState,
{
    widget: W,
    state: W::State,
}

impl<W: OverlayLayout> OverlayWrapper<W>
where
    W::State: ArstyperWidgetState,
{
    pub fn new(tx: SyncSender<UiRequest>) -> io::Result<Self> {
        Ok(Self {
            widget: W::new(tx)?,
            state: W::State::new()?,
        })
    }
}

impl<W: OverlayLayout> ArstyperOverlay for OverlayWrapper<W>
where
    W::State: ArstyperWidgetState,
{
    fn render_ref_overlay(&mut self, area: Rect, buf: &mut Buffer) {
        let overlay_area = W::overlay_area(area);
        self.widget.render_ref(overlay_area, buf, &mut self.state);
    }

    fn handle_events(&mut self, key: KeyEvent) {
        self.widget.handle_events(key, &mut self.state);
    }
}
