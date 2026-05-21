//! Menu bar screen
use crate::{
    traits::ArstyperScreen,
    ui::{Overlay, Screen, Styles, UiRequest},
};

use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, Padding, Paragraph,
        Widget, Wrap,
    },
};
use std::{
    cmp::{max, min},
    rc::Rc,
    sync::mpsc::SyncSender,
};

// display name, action, metadata
type Setting = (&'static str, UiRequest, &'static str);

const N_SETTINGS: usize = 3;
const SETTINGS: [Setting; N_SETTINGS] = [
    (
        "Close Menu",
        UiRequest::ShowOverlay(Overlay::None),
        "cm,esc",
    ),
    (
        "View Help",
        UiRequest::ChangeScreen(Screen::AboutScreen),
        "vh,help,?????,manual,info,assistance,f1",
    ),
    ("Exit arstyper", UiRequest::Exit, "quit,stop,exit,close"),
];

/// Results screen
pub struct MenuBar {
    index: usize,
    query: String,
    styles: Rc<Styles>,
    tx: SyncSender<UiRequest>,
}

fn filter_settings(query: &str) -> Vec<(&'static str, usize)> {
    let mut ret: Vec<(&str, usize)> = Vec::new();
    let _cnt = 0;
    for (i, s) in SETTINGS.iter().enumerate() {
        if s.0.to_lowercase().contains(query) || s.2.contains(query) {
            ret.push((s.0, i));
        }
    }
    return ret;
}

impl ArstyperScreen for MenuBar {
    fn new(s: Rc<Styles>, tx: SyncSender<UiRequest>) -> Self {
        Self {
            index: 0,
            query: String::with_capacity(10),
            styles: s,
            tx: tx,
        }
    }

    fn render(&self, area: Rect, buf: &mut Buffer) {
        use Constraint::{Length, Min};
        // area
        Clear.render(area, buf);
        let b = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .style(self.styles.accent)
            .padding(Padding::horizontal(1));
        let bi = b.inner(area);
        b.render(area, buf);

        let [list_a, search_a] = Layout::vertical([Min(1), Length(1)]).areas(bi);

        // list
        let items = filter_settings(&self.query);
        let n: usize = list_a.height.into();
        let start: usize = (min(max(items.len(), n) - n, max(n / 2, self.index) - n / 2)).into();
        let end: usize = (start + n).into();
        let lines = items
            .iter()
            .enumerate()
            .filter_map(|(i, x)| {
                if i == self.index {
                    Some(Line::from(Span::raw(x.0).style(self.styles.accent_inv)))
                } else if i >= start && i < end {
                    Some(Line::from(x.0))
                } else {
                    None
                }
            })
            .collect::<Vec<Line>>();
        Paragraph::new(lines)
            .style(self.styles.root)
            .wrap(Wrap { trim: false })
            .render(list_a, buf);

        // search
        Line::from(vec![
            Span::raw("> "),
            if self.query.len() == 0 {
                Span::styled("type to search", self.styles.root).italic()
            } else {
                Span::raw(self.query.clone())
            },
        ])
        .style(self.styles.accent)
        .render(search_a, buf);
    }

    fn handle_events(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(c) => {
                self.index = 0;
                self.query.push(c);
            }
            KeyCode::Backspace => {
                self.query.pop();
            }
            KeyCode::Up => {
                self.index = max(1, self.index) - 1;
            }
            KeyCode::Down => {
                self.index = min(self.index + 1, N_SETTINGS); // TODO annoying and stuff
            }
            KeyCode::Enter => {
                let items = filter_settings(&self.query);
                if items.len() >= self.index {
                    self.tx
                        .send(SETTINGS[items[self.index].1].1.clone())
                        .unwrap();
                }
                self.tx.send(UiRequest::ShowOverlay(Overlay::None)).unwrap();
            }
            _ => {}
        }
    }
}
