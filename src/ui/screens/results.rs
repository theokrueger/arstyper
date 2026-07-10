//! Results screen
use crate::{
    globs,
    traits::{ArstyperWidget, ArstyperWidgetState},
    ui::{Screen, UiRequest},
};

use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent},
    layout::{
        Constraint::{Length, Min, Percentage},
        Layout, Rect,
    },
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, StatefulWidgetRef, Widget},
};
use std::{
    cmp::{max, min},
    io,
    sync::mpsc::SyncSender,
};

#[derive(Default)]
/// Results state
pub struct ResultsState {
    speed: f32,
    last_speed: f32,

    raw_speed: f32,
    last_raw_speed: f32,

    completion: f32,
    last_completion: f32,

    acc: f32,
    last_acc: f32,

    valid: bool,
    last_valid: bool,

    show_diff: bool,

    h_scroll: usize,
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
        self.last_completion = self.completion;
        self.last_valid = self.valid;

        let scoremgr = globs::scoremgr();
        self.speed = scoremgr.score.speed(false);
        self.raw_speed = scoremgr.score.speed(true);
        self.acc = scoremgr.score.accuracy();
        self.completion = scoremgr.score.completion();
        self.valid = scoremgr.score.valid();

        self.show_diff = self.valid && self.last_valid;
    }

    /// Comparison to last test
    /// ret: `(speed, raw_speed, acc, peak_speed)`
    fn delta_stats(&self) -> (f32, f32, f32, f32) {
        (
            self.speed - self.last_speed,
            self.raw_speed - self.last_raw_speed,
            (self.acc - self.last_acc) / self.last_acc,
            0.0, // TODO
        )
    }

    /// Speed as WPM/CPM
    fn speed_span(&self, raw: bool, show_unit: bool) -> Span<'_> {
        Span::raw(format!(
            "{:.02}{}",
            if raw { self.raw_speed } else { self.speed },
            if show_unit {
                format!(" {}", Self::unit())
            } else {
                "".to_string()
            }
        ))
    }

    const WPM: &str = "WPM";
    const CPM: &str = "CPM";
    /// str unit wpm/cpm
    fn unit() -> &'static str {
        if globs::cfg().locale.cpm_over_wpm {
            Self::CPM
        } else {
            Self::WPM
        }
    }

    /// Accuracy as %
    fn accuracy_span(&self) -> Span<'_> {
        Span::raw(format!("{:2.02}%", self.acc))
    }

    /// Test completion as %
    fn completion_span(&self) -> Span<'_> {
        Span::raw(format!("{:2.02}%", self.completion))
    }

    fn next_h(&mut self) {
        self.h_scroll = min(2, self.h_scroll + 1);
        if !self.show_diff && self.h_scroll == 1 {
            self.h_scroll = 2
        }
    }

    fn prev_h(&mut self) {
        self.h_scroll = max(0, self.h_scroll.saturating_sub(1));
        if !self.show_diff && self.h_scroll == 1 {
            self.h_scroll = 0
        }
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

    fn handle_events(&mut self, key: KeyEvent, state: &mut Self::State) {
        match key.code {
            KeyCode::Left => state.prev_h(),
            KeyCode::Right => state.next_h(),
            KeyCode::Enter => {
                self.tx.send(UiRequest::ClearStatus).unwrap();
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
        let small = (area.width.saturating_sub(2)) / 3 < 20;

        // content
        let [text_a, footer_a] = Layout::vertical([Min(0), Length(1 + small as u16)]).areas(area);
        let b = Block::new()
            .borders(Borders::TOP)
            .style(globs::sty().accent)
            .title("Results & Analysis".bold())
            .padding(Padding::horizontal(1));
        let b_a = b.inner(text_a);
        b.render(text_a, buf);

        let [_graph_a, overview_a] = Layout::vertical([Min(0), Length(6)]).areas(b_a);
        let [stats_a, comp_a, info_a] = Layout::horizontal([
            Percentage(33 + 17 * !state.show_diff as u16),
            Percentage(33 - 33 * !state.show_diff as u16),
            Percentage(33 + 17 * !state.show_diff as u16),
        ])
        .areas(overview_a);

        let base_block = Block::new()
            .borders(Borders::ALL)
            .style(globs::sty().accent);
        // stats
        let stats_block = base_block.clone().title("Metrics");
        let stats_text = vec![
            Line::from(state.speed_span(false, true).bold()),
            Line::from(vec![Span::raw("Raw: "), state.speed_span(true, true)])
                .style(globs::sty().dark_text),
            Line::from(vec![
                Span::styled("Acc%: ", globs::sty().accent),
                state.accuracy_span().bold(),
            ]),
            Line::from(vec![Span::raw("Prog: "), state.completion_span()])
                .style(globs::sty().dark_text),
        ];

        let stats_p = Paragraph::new(stats_text)
            .style(globs::sty().root)
            .block(stats_block);

        // comparison
        let (d_speed, d_raw_speed, d_acc, d_peak_speed) = state.delta_stats();

        // span of sign +/- and color for a f32 stat
        macro_rules! sign_span {
            ($stat:ident) => {
                if $stat > 0.01 {
                    Span::styled(format!("+{:.02}", $stat), globs::sty().accent)
                } else if $stat < -0.01 {
                    Span::styled(format!("{:.02}", $stat), globs::sty().incorrect)
                } else {
                    Span::styled(format!("{:.02}", $stat), globs::sty().dark_text)
                }
            };
        }

        let comp_block = base_block.clone().title("Comparison");
        let comp_text = vec![
            Line::from(vec![
                Span::styled(format!("Δ{}: ", ResultsState::unit()), globs::sty().accent),
                sign_span!(d_speed),
            ]),
            Line::from(vec![
                Span::styled("ΔRaw: ", globs::sty().dark_text),
                sign_span!(d_raw_speed),
            ]),
            Line::from(vec![
                Span::styled("ΔAcc: ", globs::sty().accent),
                sign_span!(d_acc),
            ]),
            Line::from(vec![
                Span::styled("Peak: ", globs::sty().dark_text),
                sign_span!(d_peak_speed),
            ]),
        ];
        let comp_p = Paragraph::new(comp_text)
            .style(globs::sty().root)
            .block(comp_block);

        // info
        let info_block = base_block.clone().title("Test".bold());
        let info_text = vec![
            Line::from(Span::styled("TODO", globs::sty().root).bold()),
            Line::from(vec![
                Span::styled("Type: ", globs::sty().accent),
                Span::raw("TODO"),
            ]),
            Line::from(vec![Span::raw("Completed: "), Span::raw("TODO")])
                .style(globs::sty().dark_text),
            Line::from(vec![Span::raw("Time: "), Span::raw("TODO")]).style(globs::sty().dark_text),
        ];

        let info_p = Paragraph::new(info_text)
            .style(globs::sty().root)
            .block(info_block);

        // render one at a time IFF small mode is on
        if small {
            match state.h_scroll {
                0 => stats_p,
                1 => comp_p,
                2 => info_p,
                _ => unreachable!(),
            }
            .render(overview_a, buf);
        } else {
            stats_p.render(stats_a, buf);
            comp_p.render(comp_a, buf);
            info_p.render(info_a, buf);
        }

        // footer
        Paragraph::new(vec![
            Line::from("Press 'Enter' to restart or 'q' to go back."),
            Line::from("Use ⭠/⭢ to swap between metrics."),
        ])
        .style(globs::sty().accent)
        .centered()
        .render(footer_a, buf);
    }
}
