use crate::{
    config, globs_ref,
    lang::Lang,
    sty,
    traits::{ArstyperOverlay, ArstyperWidget, ArstyperWidgetState},
    ui::{
        AppState, Overlay, Screen, UiRequest,
        overlays::menu::MenuOverlay,
        screens::{
            about::{About, AboutState},
            results::{Results, ResultsState},
            stats::{Stats, StatsState},
            test::{Test, TestState},
        },
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
    time::Duration,
};

/// Main UI widget
pub struct Main {
    tick_duration: Duration,
}

impl Main {
    pub fn new() -> Self {
        Self {
            tick_duration: Duration::from_micros(
                (1.0 / config!(ui.framerate) * 1000.0 * 1000.0) as u64,
            ),
        }
    }

    /// 'tick' to update state/logic of ui
    pub fn tick(&self, state: &mut MainState<'static>) -> io::Result<()> {
        self.handle_events(state)?;

        // non-event-driven state logic
        let t = Local::now();
        if t >= state.clear_status_at {
            state.clear_status();
        }

        // message handling
        while let Ok(msg) = state.uireq_rx.try_recv() {
            match msg {
                UiRequest::Empty => {}
                UiRequest::UpdateResults => state.results_state.update(),
                UiRequest::Exit => state.state = AppState::Stopped,
                UiRequest::ChangeScreen(s) => state.change_screen(s),
                UiRequest::ClearStatus => state.clear_status(),
                UiRequest::GoToLastScreen => state.change_screen(state.last_screen.clone()),
                UiRequest::AddOverlay(o) => state.add_overlay(o)?,
                UiRequest::RemoveOverlay => {
                    state.overlay_stack.pop();
                }
                UiRequest::NewTest => state.new_test(),
            }
        }

        Ok(())
    }

    /// handle keyboard events
    pub fn handle_events(&self, state: &mut MainState<'static>) -> io::Result<()> {
        if event::poll(self.tick_duration)?
            && let Event::Key(key) = event::read()?
        {
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
                        if state.overlay_stack.len() <= 0 {
                            state.change_screen(Screen::AboutScreen);
                            pass_from_global = false;
                        }
                    }
                    KeyCode::Esc => {
                        if state.overlay_stack.len() <= 0 {
                            state
                                .uireq_tx
                                .send(UiRequest::AddOverlay(Overlay::Menu))
                                .unwrap();
                        } else {
                            state.overlay_stack.pop().unwrap();
                        };
                        pass_from_global = false;
                    }
                    _ => {}
                }

                if !pass_from_global {
                    return Ok(());
                }

                // overlay takes precedent over screen
                if state.overlay_stack.len() <= 0 {
                    match state.screen {
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
                    };
                } else {
                    state.overlay_stack.last_mut().unwrap().handle_events(key);
                }
            }
        }
        Ok(())
    }
}

impl StatefulWidgetRef for Main {
    type State = MainState<'static>;
    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if area.width < 30 || area.height < 15 {
            Paragraph::new("Terminal size too small for arstyper! Minimum w=30 h=15 chars.")
                .wrap(Wrap { trim: true })
                .render(area, buf);
            return;
        }

        use Constraint::{Length, Min};
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

        state
            .overlay_stack
            .iter_mut()
            .for_each(|x| x.render_ref_overlay(area, buf));

        state.render_modeline(mode_a, buf);
        state.render_status(status_a, buf);
    }
}

impl StatefulWidget for &Main {
    type State = MainState<'static>;
    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        self.render_ref(area, buf, state);
    }
}

/// State of main UI and all subwidgets
pub struct MainState<'a> {
    lang: Lang,

    pub state: AppState,
    screen: Screen,
    last_screen: Screen,
    overlay_stack: Vec<Box<dyn ArstyperOverlay>>,

    test: Test,
    test_state: TestState<'a>,

    stats: Stats,
    stats_state: StatsState,

    results: Results,
    results_state: ResultsState,

    about: About,
    about_state: AboutState,

    status: String,

    /// When the status message is to be cleared
    clear_status_at: DateTime<Local>,

    // communication between screens and stuff
    uireq_tx: SyncSender<UiRequest>,
    uireq_rx: Receiver<UiRequest>,
}

impl MainState<'_> {
    pub fn new() -> Result<Self, std::io::Error> {
        let lang = Lang::get_by_name(globs_ref!(cfg.lang))?;

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

            state: AppState::default(),
            screen: Screen::default(),
            last_screen: Screen::default(),
            overlay_stack: Vec::new(),

            status: if config!(ui.show_welcome_message) {
                "Welcome to arstyper! Press <F1> for help, or <Ctrl-C> to exit.".to_string()
            } else {
                "".to_string()
            },
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
                if config!(locale.show_24_hour_time) {
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

    pub fn add_overlay(&mut self, o: Overlay) -> io::Result<()> {
        let s: Option<Box<dyn ArstyperOverlay>> = Some(Box::new(match o {
            Overlay::Menu => MenuOverlay::new(self.uireq_tx.clone())?,
        }));
        self.overlay_stack.push(s.unwrap());
        Ok(())
    }
}
