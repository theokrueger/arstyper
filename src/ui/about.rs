//! "About this program" screen
use crate::{
    traits::ArstyperScreen,
    ui::{Screen, Styles, UiRequest},
};

use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{
        Constraint::{Length, Min},
        Layout, Rect,
    },
    prelude::StatefulWidget,
    style::Stylize,
    text::Line,
    widgets::{
        Block, Borders, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
        Widget, Wrap,
    },
};
use std::{rc::Rc, sync::mpsc::SyncSender};

const ABOUT_TEXT: &str = include_str!("./ABOUT.txt"); // easier to write there than here

/// About screen
pub struct About {
    scroll: ScrollbarState,
    styles: Rc<Styles>,
    tx: SyncSender<UiRequest>,
}

impl ArstyperScreen for About {
    fn new(s: Rc<Styles>, tx: SyncSender<UiRequest>) -> Self {
        Self {
            scroll: ScrollbarState::new(ABOUT_TEXT.lines().count()),
            styles: s,
            tx: tx,
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        let [body_a, footer_a] = Layout::vertical([Min(0), Length(1)]).areas(area);
        let [text_a, scrollbar_a] = Layout::horizontal([Min(0), Length(1)]).areas(body_a);
        // body text
        Paragraph::new(ABOUT_TEXT)
            .style(self.styles.root)
            .block(
                Block::new()
                    .borders(Borders::TOP)
                    .style(self.styles.accent)
                    .title(format!("{}", Screen::AboutScreen).bold())
                    .padding(Padding::horizontal(1)),
            )
            .wrap(Wrap { trim: true })
            .render(text_a, buf);

        // scrollbar
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .style(self.styles.root)
            .render(
                scrollbar_a,
                buf,
                &mut ScrollbarState::new(ABOUT_TEXT.lines().count()),
            );

        // footer
        Line::from("Press <Up>/<Down> to scroll or 'q' to go back.")
            .style(self.styles.root)
            .centered()
            .render(footer_a, buf);
    }

    fn handle_events(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') => {
                self.tx.send(UiRequest::GoToLastScreen).unwrap();
            }
            KeyCode::Up => self.scroll.prev(),
            KeyCode::Down => self.scroll.next(),
            _ => {}
        }
    }
}
