pub mod color_preview;

use crate::{config::Config, lang::Lang};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{Block, Paragraph, Tabs, Widget},
};
use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};

pub struct Ui {
    cfg: Config,
    state: State,
    screen: Screen,
    status: String,
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
    #[strum(to_string = "Test [F1]")]
    TestScreen,
    #[strum(to_string = "Results [F2]")]
    ResultsScreen,
    #[strum(to_string = "Statistics [F3]")]
    StatisticsScreen,
    #[strum(to_string = "About [F4]")]
    AboutScreen,
}

impl Ui {
    pub fn new(cfg: Config) -> Self {
        Self {
            cfg: cfg,
            state: State::default(),
            screen: Screen::default(),
            status: "asdfghjk".to_string(),
        }
    }

    pub fn run(mut self) -> std::io::Result<()> {
        let mut lang = Lang::get_by_name(&self.cfg.lang);
        let mut terminal = ratatui::init();
        while self.state != State::Stopped {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;
            self.handle_events()?;
        }

        ratatui::restore();

        Ok(())
    }

    fn handle_events(&mut self) -> std::io::Result<()> {
        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => self.state = State::Stopped,
                    KeyCode::F(1) => self.screen = Screen::TestScreen,
                    KeyCode::F(2) => self.screen = Screen::ResultsScreen,
                    KeyCode::F(3) => self.screen = Screen::StatisticsScreen,
                    KeyCode::F(4) => self.screen = Screen::AboutScreen,
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn render_tabs(&self, area: Rect, buf: &mut Buffer) {
        let titles = Screen::iter().map(|t| format!("{t}"));
        Tabs::new(titles)
            .select(self.screen.clone() as usize)
            .padding(" ", " ")
            .divider("|")
            .render(area, buf);
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
        Paragraph::new("about").render(area, buf);
    }

    fn render_status(&self, area: Rect, buf: &mut Buffer) {
        Line::raw(&self.status).render(area, buf);
    }
}

impl Widget for &Ui {
    fn render(self, area: Rect, buf: &mut Buffer) {
        use Constraint::{Length, Min, Percentage};
        let vertical = Layout::vertical([Length(1), Min(0), Length(1)]);
        let [tabs_a, body_a, status_a] = vertical.areas(area);

        self.render_tabs(tabs_a, buf);
        match self.screen {
            Screen::TestScreen => self.render_test(body_a, buf),
            Screen::ResultsScreen => self.render_results(body_a, buf),
            Screen::StatisticsScreen => self.render_statistics(body_a, buf),
            Screen::AboutScreen => self.render_about(body_a, buf),
        }
        self.render_status(status_a, buf);
    }
}
