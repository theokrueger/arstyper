//! Display all standard color combos for terminals
//! Called specially as a help argument

use crate::ui::State;
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph, Tabs, Widget},
};

struct ColorLine {
    name: String,
    fg: Color,
    is_dark: bool,
}

impl ColorLine {
    fn name_str(&self) -> &str {
        self.name.as_str()
    }
    fn fg(&self) -> Color {
        self.fg
    }
    fn bg(&self) -> Color {
        if self.is_dark {
            Color::White
        } else {
            Color::Black
        }
    }
}

macro_rules! color_line {
    ($color:expr, $isdark:expr) => {
        ColorLine {
            name: $color.to_string(),
            fg: $color,
            is_dark: $isdark,
        }
    };
}

pub struct ColorPreview {
    state: State,
    lines: Vec<ColorLine>,
    line_sel: usize,
}

const N_COLORS: usize = 16;
const N_LINES: usize = N_COLORS + 4;
impl ColorPreview {
    pub fn new() -> Self {
        Self {
            state: State::default(),
            lines: vec![
                color_line!(Color::Black, true),
                color_line!(Color::DarkGray, true),
                color_line!(Color::Red, true),
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
            line_sel: 0,
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
                    KeyCode::Char('c') => {
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            self.state = State::Stopped
                        }
                    }
                    KeyCode::Left => self.select_prev_palette(),
                    KeyCode::Right => self.select_next_palette(),
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn next_palette(&self) -> &str {
        self.lines[(self.line_sel + 1) % N_COLORS].name_str()
    }

    fn cur_palette(&self) -> &str {
        self.lines[self.line_sel].name_str()
    }

    fn prev_palette(&self) -> &str {
        let n = if (self.line_sel as i8) - 1 < 0 {
            N_COLORS - 1
        } else {
            self.line_sel - 1
        };
        self.lines[n].name_str()
    }

    fn select_next_palette(&mut self) {
        self.line_sel = (self.line_sel + 1) % N_COLORS;
    }

    fn select_prev_palette(&mut self) {
        self.line_sel = if (self.line_sel as i8) - 1 < 0 {
            N_COLORS - 1
        } else {
            self.line_sel - 1
        }
    }
}

impl Widget for &ColorPreview {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let sel = &self.lines[self.line_sel];
        let sty = Style::new().fg(sel.bg()).bg(sel.fg());
        let block = Block::bordered()
            .title_alignment(Alignment::Center)
            .style(sty)
            .title("Available colors");
        let inner_a = block.inner(area);
        block.render(area, buf);

        // footer
        use Constraint::{Length, Min, Percentage};
        let vertical = Layout::vertical([Min(0), Length(1)]);
        let [body_a, footer_a] = vertical.areas(inner_a);
        Text::from("Press ESC, q, or CTRL+C to quit.").render(footer_a, buf);

        let vertical = Layout::vertical([Constraint::Length(1); N_LINES]).split(body_a);

        // title
        {
            let [c1, c2, c3] = Layout::horizontal([Constraint::Ratio(1, 3); 3]).areas(vertical[0]);
            Text::from("Color Name").bold().centered().render(c1, buf);
            Text::from("As Foreground")
                .bold()
                .centered()
                .render(c2, buf);
            Text::from("As Background")
                .bold()
                .centered()
                .render(c3, buf);
        }
        // colors
        for (area, color_line) in vertical.iter().skip(2).take(N_COLORS).zip(&self.lines) {
            let [c1, c2, c3] = Layout::horizontal([Constraint::Ratio(1, 3); 3]).areas(*area);
            Text::from(color_line.name_str()).render(c1, buf);
            Text::from("Sample Text")
                .fg(color_line.fg())
                .render(c2, buf);
            Text::from("Sample Text")
                .fg(sel.fg())
                .bg(color_line.fg())
                .render(c3, buf);
        }
        // nav
        {
            let [c1, c2, c3] =
                Layout::horizontal([Constraint::Ratio(1, 3); 3]).areas(vertical[N_LINES - 1]);
            Text::from(format!("[{}]", self.prev_palette(),))
                .right_aligned()
                .render(c1, buf);
            Text::from(format!("<- {} ->", self.cur_palette(),))
                .centered()
                .render(c2, buf);
            Text::from(format!("[{}]", self.next_palette()))
                .left_aligned()
                .render(c3, buf);
        }
    }
}
