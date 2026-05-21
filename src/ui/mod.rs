//! Root UI
mod about;
use about::About;
pub mod color_preview;
mod results;
use results::Results;
mod test;
use test::Test;
mod stats;
use stats::Stats;
mod menubar;
use menubar::MenuBar;

use crate::{config::Config, lang::Lang, traits::ArstyperScreen};
use chrono::{DateTime, Local, TimeDelta, Timelike};
use ratatui::{
    buffer::Buffer,
    crossterm::{
        event::{
            self, Event, KeyCode, KeyEventKind, KeyModifiers, KeyboardEnhancementFlags,
            PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags, poll,
        },
        execute,
    },
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Paragraph, Widget, Wrap},
};
use std::{
    io::stdout,
    rc::Rc,
    sync::mpsc::{Receiver, SyncSender, sync_channel},
};
use strum::{Display, EnumIter, FromRepr};

/// Fat UI struct is poorly named, basically is just the whole program besides config loading.
pub struct Ui<'a> {
    cfg: Config,
    lang: Lang,

    state: AppState,
    screen: Screen,
    last_screen: Screen,
    overlay: Overlay,

    test: Test<'a>,
    results: Results,
    stats: Stats,
    about: About,

    menubar: MenuBar,

    status: String,
    /// When the status message is to be cleared
    clear_status_at: DateTime<Local>,

    /// Text and widget styles, distilled from cfg
    pub styles: Rc<Styles>,

    // communication between screens and stuff
    uireq_tx: SyncSender<UiRequest>,
    uireq_rx: Receiver<UiRequest>,
}

#[derive(Default, PartialEq)]
pub enum AppState {
    #[default]
    Running,
    Stopped,
}

#[derive(Default, Display, Clone, FromRepr, EnumIter, PartialEq)]
/// Screen to display in body area
pub enum Screen {
    #[default]
    #[strum(to_string = "Testing")]
    TestScreen,
    #[strum(to_string = "Results")]
    ResultsScreen,
    #[strum(to_string = "Stats")]
    StatsScreen,
    #[strum(to_string = "About")]
    AboutScreen,
}

#[derive(Default, PartialEq, Clone)]
/// Currenly enabled overlay
pub enum Overlay {
    #[default]
    None,
    MenuBar,
}

#[derive(Clone)]
/// Request sent by screens to here
pub enum UiRequest {
    /// Exit the program
    Exit,
    /// Change the screen (duh)
    ChangeScreen(Screen),
    /// Go to the last screen
    GoToLastScreen,
    /// Clear status
    ClearStatus,
    /// Set the overlay
    ShowOverlay(Overlay),
    //// Set the statusbar to this message. Will overwrite any existing message
    //DisplayStatus(String, DateTime<Local>),
    //// Discard current test and create a new one
    //NewTest,
}

pub struct Styles {
    pub root: Style,
    pub root_inv: Style,
    pub modeline: Style,
    pub modeline_inv: Style,
    pub accent: Style,
    pub accent_inv: Style,
    pub untyped: Style,
    pub typed: Style,
    pub incorrect: Style,
    pub cursor: Style,
}

impl Ui<'_> {
    pub fn new(cfg: Config) -> Result<Self, std::io::Error> {
        let lang = Lang::get_by_name(&cfg.lang)?;

        let root_sty = Style::new().fg(cfg.theme.fg).bg(cfg.theme.bg);
        let styles = Rc::new(Styles {
            root: root_sty,
            root_inv: root_sty.add_modifier(Modifier::REVERSED),
            modeline: root_sty.bg(cfg.theme.accent),
            modeline_inv: root_sty
                .bg(cfg.theme.accent)
                .add_modifier(Modifier::REVERSED),
            accent: root_sty.fg(cfg.theme.accent),
            accent_inv: root_sty
                .fg(cfg.theme.accent)
                .add_modifier(Modifier::REVERSED),
            untyped: root_sty.fg(cfg.theme.untyped_text),
            typed: root_sty.fg(cfg.theme.typed_text),
            incorrect: root_sty.fg(cfg.theme.incorrect_text),
            cursor: root_sty.bg(cfg.theme.accent),
        });

        let (tx, rx) = sync_channel::<UiRequest>(5);
        Ok(Self {
            styles: styles.clone(),

            test: Test::new(styles.clone(), tx.clone()),
            results: Results::new(styles.clone(), tx.clone()),
            about: About::new(styles.clone(), tx.clone()),
            stats: Stats::new(styles.clone(), tx.clone()),

            menubar: MenuBar::new(styles.clone(), tx.clone()),

            state: AppState::default(),
            overlay: Overlay::default(),
            screen: Screen::default(),
            last_screen: Screen::default(),

            status: "Welcome to arstyper! Press <F1> for help, or <Ctrl-C> to exit.".to_string(),
            clear_status_at: Local::now() + TimeDelta::seconds(5),

            cfg: cfg,
            lang: lang,

            uireq_tx: tx,
            uireq_rx: rx,
        })
    }

    pub fn run(mut self) -> std::io::Result<()> {
        let mut terminal = ratatui::init();

        // enter raw mode
        let mut stdout = stdout();
        execute!(
            stdout,
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
        )?;

        self.test
            .test_from(self.lang.gen_words(self.cfg.word_count as usize));
        self.test
            .set_title(format!("{} {}", self.lang.name, self.cfg.word_count).to_string()); // TODO use enum and strum and other things when more test types introduced
        while self.state != AppState::Stopped {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            self.handle_events()?;

            // non-event-driven state logic
            let t = Local::now();
            if t >= self.clear_status_at {
                self.clear_status();
            }

            // message handling
            while let Ok(msg) = self.uireq_rx.try_recv() {
                match msg {
                    UiRequest::Exit => self.state = AppState::Stopped,
                    UiRequest::ChangeScreen(s) => self.change_screen(s),
                    UiRequest::ClearStatus => self.clear_status(),
                    UiRequest::GoToLastScreen => self.change_screen(self.last_screen.clone()),
                    UiRequest::ShowOverlay(o) => self.overlay = o,
                }
            }
        }

        // exit raw mode
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
                            self.state = AppState::Stopped
                        }
                    }
                    KeyCode::F(1) => {
                        if self.overlay == Overlay::None {
                            self.change_screen(Screen::AboutScreen)
                        }
                    }
                    KeyCode::Esc => {
                        self.overlay = if self.overlay == Overlay::None {
                            Overlay::MenuBar
                        } else {
                            Overlay::None
                        }
                    }
                    _ => {}
                }

                // overlay takes precedent over screen
                match self.overlay {
                    Overlay::None => match self.screen {
                        Screen::AboutScreen => self.about.handle_events(key),
                        Screen::TestScreen => self.test.handle_events(key),
                        Screen::ResultsScreen => self.results.handle_events(key),
                        Screen::StatsScreen => self.stats.handle_events(key),
                    },
                    Overlay::MenuBar => self.menubar.handle_events(key),
                }
            }
        }
        Ok(())
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
        if self.screen != s {
            self.last_screen = self.screen.clone();
        }
        self.screen = s;
    }
}

impl Widget for &Ui<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if area.width < 20 || area.height < 10 {
            Paragraph::new("Terminal size too small for arstyper! Minimum w=20 h=10 chars.")
                .wrap(Wrap { trim: true })
                .render(area, buf);
            return;
        }

        use Constraint::{Length, Min, Percentage};
        let vertical = Layout::vertical([Min(0), Length(1), Length(1)]);
        let [body_a, mode_a, status_a] = vertical.areas(area);

        match self.screen {
            Screen::TestScreen => self.test.render(body_a, buf),
            Screen::ResultsScreen => self.results.render(body_a, buf),
            Screen::StatsScreen => self.stats.render(body_a, buf),
            Screen::AboutScreen => self.about.render(body_a, buf),
        }

        match self.overlay {
            Overlay::None => {}
            Overlay::MenuBar => {
                let v = Layout::vertical([Min(1), Percentage(66), Min(2)]);
                let h = Layout::horizontal([Percentage(15), Min(5), Percentage(15)]);
                let [_, mbv_a, _] = v.areas(body_a);
                let [_, mb_a, _] = h.areas(mbv_a);
                self.menubar.render(mb_a, buf);
            }
        }

        self.render_modeline(mode_a, buf);
        self.render_status(status_a, buf);
    }
}
