//! Menu screen
use crate::{
    sty,
    traits::{ArstyperOverlay, ArstyperWidget, ArstyperWidgetState},
    ui::{Overlay, Screen, UiRequest},
};

use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{Constraint, Layout, Rect},
    style::Stylize,
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, Padding, Paragraph, StatefulWidgetRef, Widget, Wrap,
    },
};
use std::{
    cmp::{max, min},
    io,
    sync::mpsc::SyncSender,
};

// display name, action, metadata
type Setting = (&'static str, UiRequest, &'static str);

const N_SETTINGS: usize = 3;
const SETTINGS: [Setting; N_SETTINGS] = [
    ("Close Menu", UiRequest::Empty, "cm,esc"),
    (
        "View Help",
        UiRequest::ChangeScreen(Screen::AboutScreen),
        "vh,help,?????,manual,info,assistance,f1",
    ),
    ("Exit arstyper", UiRequest::Exit, "quit,stop,exit,close"),
];

/// Menu overlay
pub struct MenuOverlay {
    renderable: Menu,
    state: MenuState,
}

impl MenuOverlay {
    pub fn new(tx: SyncSender<UiRequest>) -> io::Result<Self> {
        Ok(Self {
            renderable: Menu::new(tx)?,
            state: MenuState::new()?,
        })
    }
}

impl ArstyperOverlay for MenuOverlay {
    fn render_ref_overlay(&mut self, area: Rect, buf: &mut Buffer) {
        use Constraint::{Length, Min, Percentage};
        let v = Layout::vertical([Min(1), Percentage(66), Min(2)]);
        let h = Layout::horizontal([Percentage(15), Min(5), Percentage(15)]);
        let [_, v_a, _] = v.areas(area);
        let [_, a, _] = h.areas(v_a);
        self.renderable.render_ref(a, buf, &mut self.state);
    }

    fn handle_events(&mut self, key: KeyEvent) {
        self.renderable.handle_events(key, &mut self.state);
    }
}

struct Menu {
    tx: SyncSender<UiRequest>,
}

impl ArstyperWidget for Menu {
    fn new(tx: SyncSender<UiRequest>) -> io::Result<Self> {
        Ok(Self { tx: tx })
    }

    fn handle_events(&mut self, key: KeyEvent, state: &mut Self::State) {
        match key.code {
            KeyCode::Char(c) => {
                state.index = 0;
                state.query.push(c);
                state.update_filtered_subset();
            }
            KeyCode::Backspace => {
                state.query.pop();
                state.update_filtered_fresh();
            }
            KeyCode::Up => {
                state.index = max(1, state.index) - 1;
            }
            KeyCode::Down => {
                state.index = min(state.index + 1, state.filtered.len() - 1);
            }
            KeyCode::Enter => {
                if state.filtered.len() >= state.index {
                    self.tx
                        .send(SETTINGS[state.filtered[state.index].1].1.clone())
                        .unwrap();
                }
                self.tx.send(UiRequest::RemoveOverlay).unwrap();
            }
            _ => {}
        }
    }
}

impl StatefulWidgetRef for Menu {
    type State = MenuState;
    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        use Constraint::{Length, Min};
        // area
        Clear.render(area, buf);
        let b = Block::new()
            .borders(Borders::ALL)
            .border_type(BorderType::Double)
            .style(sty!(accent))
            .padding(Padding::horizontal(1));
        let bi = b.inner(area);
        b.render(area, buf);

        let [list_a, search_a] = Layout::vertical([Min(1), Length(1)]).areas(bi);

        // list
        let n: usize = list_a.height.into();
        let start: usize = (min(
            max(state.filtered.len(), n) - n,
            max(n / 2, state.index) - n / 2,
        ))
        .into();
        let end: usize = (start + n).into();
        let lines = state
            .filtered
            .iter()
            .enumerate()
            .filter_map(|(i, x)| {
                if i == state.index {
                    Some(Line::from(Span::raw(x.0).style(sty!(accent_inv))))
                } else if i >= start && i < end {
                    Some(Line::from(x.0))
                } else {
                    None
                }
            })
            .collect::<Vec<Line>>();
        Paragraph::new(lines)
            .style(sty!(root))
            .wrap(Wrap { trim: false })
            .render(list_a, buf);

        // search
        Line::from(vec![
            Span::raw("> "),
            if state.query.len() == 0 {
                Span::styled("type to search", sty!(root)).italic()
            } else {
                Span::raw(state.query.clone())
            },
        ])
        .style(sty!(accent))
        .render(search_a, buf);
    }
}

/// Menu Overlay State
pub struct MenuState {
    index: usize,
    query: String,
    filtered: Vec<(&'static str, usize)>,
}

impl MenuState {
    /// Check everything (i.e. on backspaces)
    fn update_filtered_fresh(&mut self) {
        self.filtered.clear();
        for (i, s) in SETTINGS.iter().enumerate() {
            if s.0.to_lowercase().contains(&self.query) || s.2.contains(&self.query) {
                self.filtered.push((s.0, i));
            }
        }
    }

    /// Check only currently filtered (i.e. on char typed)
    fn update_filtered_subset(&mut self) {
        self.filtered.retain(|(_, i)| {
            let s = &SETTINGS[*i];
            s.0.to_lowercase().contains(&self.query) || s.2.contains(&self.query)
        });
    }
}

impl ArstyperWidgetState for MenuState {
    fn new() -> io::Result<Self> {
        let mut s = Self {
            index: 0,
            query: String::with_capacity(10),
            filtered: Vec::with_capacity(N_SETTINGS),
        };
        s.update_filtered_fresh();
        Ok(s)
    }
}
