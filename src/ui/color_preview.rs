use crate::ui::State;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind},
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Tabs, Widget},
};

struct ColorLine {
    name: String,
    color: Color,
}

impl ColorLine {
    fn name_str(&self) -> &str {
        self.name.as_str()
    }
    fn fg(&self) -> Color {
        self.color
    }
}

macro_rules! color_line {
    ($color:expr, $isdark:expr) => {
        ColorLine {
            name: $color.to_string(),
            color: $color,
        }
    };
}

pub struct ColorPreview {
    state: State,
    lines: Vec<ColorLine>,
}

const N_COLORS: usize = 16;
impl ColorPreview {
    pub fn new() -> Self {
        Self {
            state: State::default(),
            lines: vec![
                color_line!(Color::Black, true),
                color_line!(Color::DarkGray, true),
                color_line!(Color::Red, false),
                color_line!(Color::Green, false),
                color_line!(Color::Yellow, false),
                color_line!(Color::Blue, false),
                color_line!(Color::Magenta, false),
                color_line!(Color::Cyan, false),
                color_line!(Color::Gray, false),
                color_line!(Color::LightRed, false),
                color_line!(Color::LightGreen, false),
                color_line!(Color::LightYellow, false),
                color_line!(Color::LightBlue, false),
                color_line!(Color::LightMagenta, false),
                color_line!(Color::LightCyan, false),
                color_line!(Color::White, false),
            ],
        }
    }

    pub fn run(mut self) -> std::io::Result<()> {
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
                    _ => {}
                }
            }
        }
        Ok(())
    }
}

impl Widget for &ColorPreview {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // block
        let block = Block::new()
            .borders(Borders::TOP)
            .title_alignment(Alignment::Center)
            .border_style(Style::new().gray())
            .title_style(Style::new().white())
            .title("Available colors");
        let inner_a = block.inner(area);
        block.render(area, buf);
        // footer
        use Constraint::{Length, Min, Percentage};
        let vertical = Layout::vertical([Min(0), Length(1)]);
        let [body_a, footer_a] = vertical.areas(inner_a);
        Line::raw("Press ESC, q, or CTRL+C to quit.").render(footer_a, buf);

        // colors
        let vertical = Layout::vertical([Constraint::Length(1); N_COLORS * 2 + 3]).split(body_a);
        {
            let [c1, c2, c3] = Layout::horizontal([Constraint::Ratio(1, 3); 3]).areas(vertical[0]);
            Text::from("Name").centered().render(c1, buf);
            Text::from("Text").centered().render(c2, buf);
            Text::from("Block").centered().render(c3, buf);
        }
        for (area, color_line) in vertical.iter().skip(1).zip(&self.lines) {
            let [c1, c2, c3] = Layout::horizontal([Constraint::Ratio(1, 3); 3]).areas(vertical[0]);
            Paragraph::new(color_line.name_str())
                .fg(color_line.fg())
                .bg(Color::Black())
                .render(*area, buf);
        }
    }
}
