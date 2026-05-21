//! "About this program" screen
use crate::{
    traits::{ArstyperWidget, ArstyperWidgetState},
    ui::{Styles, UiRequest},
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
        StatefulWidgetRef, Widget, Wrap,
    },
};
use std::{
    cmp::{max, min},
    io,
    rc::Rc,
    sync::mpsc::SyncSender,
};

const ABOUT_TEXT: &str = include_str!("./ABOUT.txt"); // easier to write there than here

/// About widget
pub struct About {
    styles: Rc<Styles>,
    tx: SyncSender<UiRequest>,
}

/// About widget state
pub struct AboutState {
    scroll: usize,
    max: usize,
}

impl ArstyperWidgetState for AboutState {
    fn new() -> io::Result<Self> {
        Ok(Self { scroll: 0, max: 0 })
    }
}

impl AboutState {
    fn next(&mut self, n: usize) {
        self.scroll = min(self.scroll + n, self.max);
    }

    fn prev(&mut self, n: usize) {
        self.scroll = max(self.scroll, n) - n;
    }

    fn set_max(&mut self, n: usize) {
        self.max = n;
        self.scroll = min(self.scroll, n);
    }
}

impl StatefulWidgetRef for About {
    type State = AboutState;

    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let [body_a, scrollbar_a] = Layout::horizontal([Min(0), Length(1)]).areas(area);
        let [text_a, footer_a] = Layout::vertical([Min(0), Length(1)]).areas(body_a);

        // parse text into spans n stuff
        let mut text = Vec::<Line>::with_capacity(50);
        for line in ABOUT_TEXT.lines() {
            if line.len() >= 3 && &line[0..2] == "* " {
                text.push(Line::from(line[2..].bold()));
            } else {
                text.push(Line::from(line));
            }
        }
        // body text
        let b = Block::new()
            .borders(Borders::TOP)
            .style(self.styles.accent)
            .title("About arstyper".bold())
            .padding(Padding::horizontal(1));
        let mut p = Paragraph::new(text)
            .style(self.styles.root)
            .wrap(Wrap { trim: false });

        // scrollbar
        let h = b.inner(area).height as usize;
        state.set_max(max(p.line_count(text_a.width) + 2, h) - h); // +2 for padding
        p = p.scroll((state.scroll as u16, 0)).block(b);

        let mut sbs = ScrollbarState::new(state.max).position(state.scroll);
        Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .style(self.styles.root)
            .render(scrollbar_a, buf, &mut sbs);

        p.render(text_a, buf);

        // footer
        Line::from("Use ⭡/⭣ to scroll or 'q' to go back.")
            .style(self.styles.accent)
            .centered()
            .render(footer_a, buf);
    }
}

impl ArstyperWidget for About {
    fn new(s: Rc<Styles>, tx: SyncSender<UiRequest>) -> io::Result<Self> {
        Ok(Self { styles: s, tx: tx })
    }

    fn handle_events(&mut self, key: KeyEvent, state: &mut Self::State) {
        match key.code {
            KeyCode::Char('q') => {
                self.tx.send(UiRequest::GoToLastScreen).unwrap();
            }
            KeyCode::Up => state.prev(1),
            KeyCode::Down => state.next(1),
            KeyCode::PageUp => state.prev(15),
            KeyCode::PageDown => state.next(15),
            _ => {}
        }
    }
}
