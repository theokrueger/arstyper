//! Root UI
pub mod color_preview;

use crate::{config::Config, lang::Lang, test::Test};
use chrono::{DateTime, Local, TimeDelta, Timelike};
use ratatui::{
    buffer::Buffer,
    crossterm::{
        event::{
            self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, KeyboardEnhancementFlags,
            PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags, poll,
        },
        execute,
    },
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget},
};
use std::{
    io::stdout,
    sync::mpsc::{Receiver, SyncSender, sync_channel},
};
use strum::{Display, EnumIter, FromRepr};

/// Fat UI struct is poorly named, basically is just the whole program besides config loading.
pub struct Ui<'a> {
    cfg: Config,
    lang: Lang,

    state: State,
    screen: Screen,
    last_screen: Screen,

    test: Test<'a>,

    status: String,
    /// When the status message is to be cleared
    clear_status_at: DateTime<Local>,

    /// Text and widget styles, distilled from cfg
    pub styles: Styles,

    // communication between screens and stuff
    uireq_tx: SyncSender<UiRequest>,
    uireq_rx: Receiver<UiRequest>,
}

#[derive(Default, PartialEq)]
pub enum State {
    #[default]
    Running,
    Stopped,
}

#[derive(Default, Display, Clone, FromRepr, EnumIter)]
/// Screen to display in body area
pub enum Screen {
    #[default]
    #[strum(to_string = "Testing")]
    TestScreen,
    #[strum(to_string = "Results")]
    ResultsScreen,
    #[strum(to_string = "Statistics")]
    StatisticsScreen,
    #[strum(to_string = "About")]
    AboutScreen,
}

/// Request sent by screens to here
pub enum UiRequest {
    /// Change the screen (duh)
    ChangeScreen(Screen),
    //// Set the statusbar to this message. Will overwrite any existing message
    //DisplayStatus(String, DateTime<Local>),
    //// Discard current test and create a new one
    //NewTest,
}

#[derive(Clone)]
pub struct Styles {
    pub root: Style,
    pub modeline: Style,
    pub modeline_inv: Style,
    pub accent: Style,
    pub untyped: Style,
    pub typed: Style,
    pub incorrect: Style,
    pub cursor: Style,
}

impl Ui<'_> {
    pub fn new(cfg: Config) -> Result<Self, std::io::Error> {
        let lang = Lang::get_by_name(&cfg.lang)?;

        let root_sty = Style::new().fg(cfg.theme.fg).bg(cfg.theme.bg);
        let mode_sty = root_sty.bg(cfg.theme.accent);
        let mode_inv_sty = mode_sty.add_modifier(Modifier::REVERSED);
        let accent_sty = root_sty.fg(cfg.theme.accent);
        let untyped_sty = root_sty.fg(cfg.theme.untyped_text);
        let typed_sty = root_sty.fg(cfg.theme.typed_text);
        let incorrect_sty = root_sty.fg(cfg.theme.incorrect_text);
        let cursor_sty = root_sty.bg(cfg.theme.accent);
        let styles = Styles {
            root: root_sty,
            modeline: mode_sty,
            modeline_inv: mode_inv_sty,
            accent: accent_sty,
            untyped: untyped_sty,
            typed: typed_sty,
            incorrect: incorrect_sty,
            cursor: cursor_sty,
        };

        let (tx, rx) = sync_channel::<UiRequest>(2); // 2 to avoid lockups that should never happen anyways
        Ok(Self {
            styles: styles.clone(),
            test: Test::new(styles, tx.clone()),
            state: State::default(),
            screen: Screen::default(),
            last_screen: Screen::default(),
            status: "Welcome to arstyper! Press <F1> for help, or 'Ctrl+C' to exit.".to_string(),
            clear_status_at: Local::now() + TimeDelta::seconds(5),
            cfg: cfg,
            lang: lang,
            uireq_tx: tx,
            uireq_rx: rx,
        })
    }

    pub fn run(mut self) -> std::io::Result<()> {
        let mut terminal = ratatui::init();

        let mut stdout = stdout();
        execute!(
            stdout,
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
        )?;

        self.test
            .test_from(self.lang.gen_words(self.cfg.word_count as usize));
        while self.state != State::Stopped {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            self.handle_events()?;

            // non-event-driven state logic
            let t = Local::now();
            if t >= self.clear_status_at {
                self.clear_status();
            }

            // message handling
            if let Ok(msg) = self.uireq_rx.try_recv() {
                match msg {
                    UiRequest::ChangeScreen(s) => self.screen = s,
                }
            }
        }

        execute!(stdout, PopKeyboardEnhancementFlags)?;
        ratatui::restore();

        Ok(())
    }

    fn handle_events(&mut self) -> std::io::Result<()> {
        if poll(std::time::Duration::from_secs(1))?
            && let Event::Key(key) = event::read()?
        {
            if key.kind == KeyEventKind::Press {
                // global keys
                match key.code {
                    KeyCode::Char('c') => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            self.state = State::Stopped
                        }
                    }
                    KeyCode::F(1) => {
                        self.set_status_for(
                            "Press <ESC> or 'q' to go back.".to_string(),
                            TimeDelta::seconds(3),
                        );
                        self.change_screen(Screen::AboutScreen)
                    }
                    _ => {}
                }

                // per-screen keys
                match self.screen {
                    Screen::AboutScreen => self.handle_about_events(key),
                    Screen::TestScreen => self.test.handle_events(key),
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn render_results(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("res").render(area, buf);
    }

    fn render_statistics(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("stats").render(area, buf);
    }

    fn render_about(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("arstyper by theokrueger").render(area, buf);
    }

    fn handle_about_events(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.clear_status();
                self.change_screen(self.last_screen.clone());
            }
            _ => {}
        }
    }

    fn render_modeline(&self, area: Rect, buf: &mut Buffer) {
        let [c1, time_a] =
            Layout::horizontal([Constraint::Min(0), Constraint::Length(8)]).areas(area);

        let mode = format!("{}", self.screen);
        Line::from(vec![
            Span::raw("arstyper "),
            Span::raw(mode).style(self.styles.modeline_inv),
        ])
        .style(self.styles.modeline)
        .render(c1, buf);

        let time = if self.cfg.ui.show_clock {
            let t = Local::now();
            format!(
                "{:02}:{:02}:{:02}",
                if self.cfg.ui.hour_24 {
                    t.hour()
                } else {
                    t.hour12().1
                },
                t.minute(),
                t.second()
            )
        } else {
            " ".to_string()
        };
        Line::from(time)
            .style(self.styles.modeline)
            .render(time_a, buf);
    }

    fn render_status(&self, area: Rect, buf: &mut Buffer) {
        Line::raw(&self.status)
            .style(self.styles.root)
            .render(area, buf);
    }

    fn set_status_for(&mut self, s: String, t: TimeDelta) {
        self.status = s;
        self.clear_status_at = Local::now() + t;
    }

    fn clear_status(&mut self) {
        self.status = " ".to_string(); // such that background color can be preserved
        self.clear_status_at = DateTime::<Local>::MAX_UTC.into()
    }

    fn change_screen(&mut self, s: Screen) {
        self.last_screen = self.screen.clone();
        self.screen = s;
    }
}

impl Widget for &Ui<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use Constraint::{Length, Min};
        let vertical = Layout::vertical([Min(0), Length(1), Length(1)]);
        let [body_a, mode_a, status_a] = vertical.areas(area);

        match self.screen {
            Screen::TestScreen => self.test.render(body_a, buf),
            Screen::ResultsScreen => self.render_results(body_a, buf),
            Screen::StatisticsScreen => self.render_statistics(body_a, buf),
            Screen::AboutScreen => self.render_about(body_a, buf),
        }

        self.render_modeline(mode_a, buf);
        self.render_status(status_a, buf);
    }
}
