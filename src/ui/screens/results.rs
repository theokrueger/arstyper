//! Results screen
use crate::{
    ScoreManager, globs, globs_apply, sty,
    traits::{ArstyperWidget, ArstyperWidgetState},
    ui::{Screen, UiRequest},
};

use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{
        Constraint::{Length, Min},
        Layout, Rect,
    },
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, StatefulWidgetRef, Widget, Wrap},
};
use std::{io, sync::mpsc::SyncSender};

#[derive(Default)]
/// Results state
pub struct ResultsState {
    speed: f32,
    last_speed: f32,
    raw_speed: f32,
    last_raw_speed: f32,
    acc: f32,
    last_acc: f32,
    show_diff: bool,
}

impl ArstyperWidgetState for ResultsState {
    fn new() -> io::Result<Self> {
        let mut s = Self::default();
        s.update();
        Ok(s)
    }
}

impl ResultsState {
    /// Update state from scoremanager data
    pub fn update(&mut self) {
        self.last_speed = self.speed;
        self.last_raw_speed = self.raw_speed;
        self.last_acc = self.acc;

        globs_apply!(scoremgr, |x: &ScoreManager| {
            self.speed = x.score.speed(false);
            self.raw_speed = x.score.speed(true);
            self.acc = x.score.accuracy();
        });
    }

    /// Speed as WPM/CPM
    fn speed_span(&self, raw: bool) -> Span<'_> {
        let unit = if globs!(cfg.locale.cpm_over_wpm) {
            "CPM"
        } else {
            "WPM"
        };
        Span::styled(
            format!(
                "{:.02} {unit}",
                if raw { self.raw_speed } else { self.speed }
            ),
            sty!(root),
        )
    }

    /// Accuracy
    fn acc_span(&self) -> Span<'_> {
        Span::styled(format!("{:2.02}%", self.acc), sty!(root))
    }
}

/// Results screen
pub struct Results {
    tx: SyncSender<UiRequest>,
}

impl ArstyperWidget for Results {
    fn new(tx: SyncSender<UiRequest>) -> io::Result<Self> {
        Ok(Self { tx: tx })
    }

    fn handle_events(&mut self, key: KeyEvent, _state: &mut Self::State) {
        match key.code {
            KeyCode::Enter => {
                self.tx.send(UiRequest::NewTest).unwrap();
                self.tx
                    .send(UiRequest::ChangeScreen(Screen::TestScreen))
                    .unwrap();
            }
            _ => {}
        }
    }
}

impl StatefulWidgetRef for Results {
    type State = ResultsState;
    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let [text_a, footer_a] = Layout::vertical([Min(0), Length(1)]).areas(area);

        // body text
        let b = Block::new()
            .borders(Borders::TOP)
            .style(sty!(accent))
            .title("Results & Analysis".bold())
            .padding(Padding::horizontal(1));
        let p = Paragraph::new(Line::from(vec![
            state.speed_span(false),
            state.speed_span(true),
            state.acc_span(),
        ]))
        .style(sty!(root))
        .wrap(Wrap { trim: false })
        .block(b);

        p.render(text_a, buf);

        // footer
        Line::from("Use ⭡/⭣ to scroll or 'q' to go back.")
            .style(sty!(accent))
            .centered()
            .render(footer_a, buf);
    }
}
