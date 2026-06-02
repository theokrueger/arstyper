use crate::{
    config, config_ref,
    lang::Lang,
    sty,
    traits::{ArstyperOverlay, ArstyperWidget, ArstyperWidgetState},
    ui::{
        AppState, Overlay, Screen, UiRequest,
        about::{About, AboutState},
        menubar::{MenuBar, MenuBarState},
        results::{Results, ResultsState},
        stats::{Stats, StatsState},
        test::{Test, TestState},
    },
};

use chrono::{DateTime, Local, TimeDelta, Timelike};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Constraint, Layout, Rect},
    text::{Line, Span},
    widgets::{Paragraph, StatefulWidget, StatefulWidgetRef, Widget, Wrap},
};
use std::{
    io::{self},
    sync::mpsc::{Receiver, SyncSender, sync_channel},
};

/// Main UI widget
pub struct Ui {}

impl Ui {
    pub fn new() -> Self {
        Self {}
    }

    /// 'tick' to update state/logic of ui
    pub fn tick(&self, state: &mut UiState<'static>) -> io::Result<()> {
        self.handle_events(state)?;

        // non-event-driven state logic
        let t = Local::now();
        if t >= state.clear_status_at {
            state.clear_status();
        }

        // message handling
        while let Ok(msg) = state.uireq_rx.try_recv() {
            match msg {
                UiRequest::Exit => state.state = AppState::Stopped,
                UiRequest::ChangeScreen(s) => state.change_screen(s),
                UiRequest::ClearStatus => state.clear_status(),
                UiRequest::GoToLastScreen => state.change_screen(state.last_screen.clone()),
                UiRequest::ShowOverlay(o) => state.overlay = o,
                UiRequest::NewTest => state.new_test(),
            }
        }

        Ok(())
    }

    /// handle keyboard events
    pub fn handle_events(&self, state: &mut UiState<'static>) -> io::Result<()> {
        // if poll(Duration::from_secs(1))?
        if let Event::Key(key) = event::read()? {
            let mut pass_from_global = true;
            if key.kind == KeyEventKind::Press {
                // global keys
                match key.code {
                    KeyCode::Char('c') => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            state.state = AppState::Stopped;
                            pass_from_global = false;
                        }
                    }
                    KeyCode::F(1) => {
                        if state.overlay == Overlay::None {
                            state.change_screen(Screen::AboutScreen);
                            pass_from_global = false;
                        }
                    }
                    KeyCode::Esc => {
                        state.overlay = if state.overlay == Overlay::None {
                            Overlay::MenuBar
                        } else {
                            Overlay::None
                        };
                        pass_from_global = false;
                    }
                    _ => {}
                }

                if !pass_from_global {
                    return Ok(());
                }
                // overlay takes precedent over screen
                match state.overlay {
                    Overlay::None => match state.screen {
                        Screen::AboutScreen => {
                            state.about.handle_events(key, &mut state.about_state)
                        }
                        Screen::TestScreen => state.test.handle_events(key, &mut state.test_state),
                        Screen::ResultsScreen => {
                            state.results.handle_events(key, &mut state.results_state)
                        }
                        Screen::StatsScreen => {
                            state.stats.handle_events(key, &mut state.stats_state)
                        }
                    },
                    Overlay::MenuBar => state.menubar.handle_events(key, &mut state.menubar_state),
                }
            }
        }
        Ok(())
    }
}

impl StatefulWidgetRef for Ui {
    type State = UiState<'static>;
    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if area.width < 20 || area.height < 10 {
            Paragraph::new("Terminal size too small for arstyper! Minimum w=20 h=10 chars.")
                .wrap(Wrap { trim: true })
                .render(area, buf);
            return;
        }

        use Constraint::{Length, Min, Percentage};
        let vertical = Layout::vertical([Min(0), Length(1), Length(1)]);
        let [body_a, mode_a, status_a] = vertical.areas(area);

        match state.screen {
            Screen::TestScreen => state.test.render_ref(body_a, buf, &mut state.test_state),
            Screen::ResultsScreen => {
                state
                    .results
                    .render_ref(body_a, buf, &mut state.results_state)
            }
            Screen::StatsScreen => state.stats.render_ref(body_a, buf, &mut state.stats_state),
            Screen::AboutScreen => state.about.render_ref(body_a, buf, &mut state.about_state),
        }

        match state.overlay {
            Overlay::None => {}
            Overlay::MenuBar => {
                state
                    .menubar
                    .render_ref_overlay(area, buf, &mut state.menubar_state);
            }
        }

        state.render_modeline(mode_a, buf);
        state.render_status(status_a, buf);
    }
}

impl StatefulWidget for &Ui {
    type State = UiState<'static>;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.render_ref(area, buf, state);
    }
}

/// State of main UI and all subwidgets
pub struct UiState<'a> {
    pub lang: Lang,

    pub state: AppState,
    pub screen: Screen,
    pub last_screen: Screen,
    pub overlay: Overlay,

    pub test: Test,
    pub test_state: TestState<'a>,

    pub stats: Stats,
    pub stats_state: StatsState,

    pub results: Results,
    pub results_state: ResultsState,

    pub about: About,
    pub about_state: AboutState,

    pub menubar: MenuBar,
    pub menubar_state: MenuBarState,

    pub status: String,
    /// When the status message is to be cleared
    pub clear_status_at: DateTime<Local>,

    // communication between screens and stuff
    pub uireq_tx: SyncSender<UiRequest>,
    pub uireq_rx: Receiver<UiRequest>,
}

impl UiState<'_> {
    pub fn new() -> Result<Self, std::io::Error> {
        let lang = Lang::get_by_name(config_ref!(lang))?;

        let (tx, rx) = sync_channel::<UiRequest>(5);
        let mut ret = Self {
            test: Test::new(tx.clone())?,
            test_state: TestState::new()?,

            stats: Stats::new(tx.clone())?,
            stats_state: StatsState::new()?,

            results: Results::new(tx.clone())?,
            results_state: ResultsState::new()?,

            about: About::new(tx.clone())?,
            about_state: AboutState::new()?,

            menubar: MenuBar::new(tx.clone())?,
            menubar_state: MenuBarState::new()?,

            state: AppState::default(),
            overlay: Overlay::default(),
            screen: Screen::default(),
            last_screen: Screen::default(),

            status: "Welcome to arstyper! Press <F1> for help, or <Ctrl-C> to exit.".to_string(),
            clear_status_at: Local::now() + TimeDelta::seconds(5),

            lang: lang,

            uireq_tx: tx,
            uireq_rx: rx,
        };

        ret.new_test();

        Ok(ret)
    }

    fn new_test(&mut self) {
        self.test_state
            .test_from(self.lang.gen_words(config!(word_count) as usize));
        self.test_state
            .set_title(format!("{} {}", self.lang.name, config!(word_count)).to_string()); // TODO use enum and strum and other things when more test types introduced
    }

    pub fn render_modeline(&self, area: Rect, buf: &mut Buffer) {
        let [c1, time_a] =
            Layout::horizontal([Constraint::Min(0), Constraint::Length(5)]).areas(area);
        let mode = format!("{}", self.screen);
        Line::from(vec![
            Span::raw("arstyper "),
            Span::raw(mode).style(sty!(modeline_inv)),
        ])
        .style(sty!(modeline))
        .render(c1, buf);

        let time = if config!(ui.show_clock) {
            let t = Local::now();
            format!(
                "{:02}:{:02}",
                if config!(ui.hour_24) {
                    t.hour()
                } else {
                    t.hour12().1
                },
                t.minute()
            )
        } else {
            " ".to_string()
        };
        Line::from(time).style(sty!(modeline)).render(time_a, buf);
    }

    pub fn render_status(&self, area: Rect, buf: &mut Buffer) {
        Line::raw(&self.status).style(sty!(root)).render(area, buf);
    }

    pub fn set_status_for(&mut self, s: String, t: TimeDelta) {
        self.status = s;
        self.clear_status_at = Local::now() + t;
    }

    pub fn clear_status(&mut self) {
        self.status = " ".to_string(); // such that background color can be preserved
        self.clear_status_at = DateTime::<Local>::MAX_UTC.into()
    }

    pub fn change_screen(&mut self, s: Screen) {
        if self.screen != s {
            self.last_screen = self.screen.clone();
        }
        self.screen = s;
    }
}
