//! Root UI
pub mod color_preview;

use crate::{config::Config, lang::Lang, test::Test};
use chrono::{DateTime, Local, TimeDelta, Timelike};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Paragraph, Tabs, Widget},
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};

pub struct Ui {
    cfg: Config,
    lang: Lang,

    state: State,
    screen: Screen,
    last_screen: Screen,

    status: String,
    clear_status_at: DateTime<Local>,

    styles: Styles,
}

#[derive(Default, PartialEq)]
pub enum State {
    #[default]
    Running,
    Stopped,
}

#[derive(Default, Display, Clone, FromRepr, EnumIter)]
enum Screen {
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

struct Styles {
    root: Style,
    modeline: Style,
    modeline_inv: Style,
}

impl Ui {
    pub fn new(cfg: Config) -> Self {
        let lang = Lang::get_by_name(&cfg.lang);

        let root_sty = Style::new().fg(cfg.theme.fg).bg(cfg.theme.bg);
        let mode_sty = root_sty.bg(cfg.theme.accent);
        let mode_inv_sty = mode_sty.add_modifier(Modifier::REVERSED);
        Self {
            styles: Styles {
                root: root_sty,
                modeline: mode_sty,
                modeline_inv: mode_inv_sty,
            },
            state: State::default(),
            screen: Screen::default(),
            last_screen: Screen::default(),
            status: "Welcome to arstyper! Press <F1> for help, or 'Ctrl+C' to exit.".to_string(),
            clear_status_at: Local::now() + TimeDelta::seconds(3),
            cfg: cfg,
            lang: lang,
        }
    }

    pub fn run(mut self) -> std::io::Result<()> {
        let mut terminal = ratatui::init();
        while self.state != State::Stopped {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            self.handle_events()?;

            // non-event-driven state logic
            // (mostly things i'm to lazy to spin a thread up for)
            let t = Local::now();
            if t >= self.clear_status_at {
                self.clear_status();
            }
            match self.screen {
                //                Screen::TestScreen => self.test,
                _ => {}
            }
        }

        ratatui::restore();

        Ok(())
    }

    fn handle_events(&mut self) -> std::io::Result<()> {
        if let Event::Key(key) = event::read()? {
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
                    Screen::AboutScreen => self.handle_about_events(key.code),
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn render_test(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new("test").render(area, buf);
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

    fn handle_about_events(&mut self, kc: KeyCode) {
        match kc {
            KeyCode::Esc | KeyCode::Char('q') => {
                self.clear_status();
                self.change_screen(self.last_screen.clone());
            }
            _ => {}
        }
    }

    fn render_modeline(&self, area: Rect, buf: &mut Buffer) {
        let [c1, time_a] =
            Layout::horizontal([Constraint::Min(0), Constraint::Length(5)]).areas(area);

        let mode = format!("{}", self.screen);
        Line::from(vec![
            Span::raw("arstyper "),
            Span::raw(mode).style(self.styles.modeline_inv),
        ])
        .style(self.styles.modeline)
        .render(c1, buf);

        let time = if self.cfg.ui.show_clock {
            let t = Local::now();
            format!("{:02}:{:02}", t.hour(), t.minute())
        } else {
            "".to_string()
        };
        Span::raw(time)
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
        self.status = "".to_string();
        self.clear_status_at = DateTime::<Local>::MAX_UTC.into()
    }

    fn change_screen(&mut self, s: Screen) {
        self.last_screen = self.screen.clone();
        self.screen = s;
    }
}

impl Widget for &Ui {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use Constraint::{Length, Min, Percentage};
        let vertical = Layout::vertical([Min(0), Length(1), Length(1)]);
        let [body_a, mode_a, status_a] = vertical.areas(area);

        match self.screen {
            Screen::TestScreen => self.render_test(body_a, buf),
            Screen::ResultsScreen => self.render_results(body_a, buf),
            Screen::StatisticsScreen => self.render_statistics(body_a, buf),
            Screen::AboutScreen => self.render_about(body_a, buf),
        }

        self.render_modeline(mode_a, buf);
        self.render_status(status_a, buf);
    }
}
