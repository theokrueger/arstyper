//! Typing test struct
use crate::ui::Ui;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text, ToLine},
    widgets::{Block, Borders, Padding, Paragraph, Tabs, Widget, Wrap},
};
use std::{cmp::min, time::Instant};

const BKSPC: char = '';

/// A single keypress
struct Keypress {
    key: char,
    time: Instant,
}

impl Keypress {
    /// Create keypress from char with current time as instant
    fn from_chr(key: char) -> Self {
        Self {
            key: key,
            time: Instant::now(),
        }
    }
}

/// A single test word and its keypresses.
struct TestWord {
    word: String,
    presses: Vec<Keypress>,
    correct: bool,
}

impl From<String> for TestWord {
    fn from(string: String) -> Self {
        TestWord {
            presses: Vec::with_capacity(string.len()),
            word: string,
            correct: false,
        }
    }
}

/// The actual typing test
pub struct Test {
    words: Vec<TestWord>,
    word_i: usize,
}

impl Test {
    /// Create a new emtpy test, which must be initialised before use :D
    pub fn new() -> Self {
        Test {
            words: Vec::new(),
            word_i: 0,
        }
    }

    /// Handle keypress events for this test
    pub fn handle_events(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(' ') => {
                self.word_i += 1;
            }
            KeyCode::Char(chr) => self.words[self.word_i]
                .presses
                .push(Keypress::from_chr(chr)),
            KeyCode::Backspace => self.words[self.word_i]
                .presses
                .push(Keypress::from_chr(BKSPC)),
            _ => {}
        }
    }

    /// Create test from an iterator over string items
    pub fn test_from(&mut self, words: impl Iterator<Item = String>) {
        self.words = words
            .map(|w| w.to_lowercase().into())
            .collect::<Vec<TestWord>>();
    }

    /// Render the test text
    pub fn render(&self, ui: &Ui, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.words_to_line(ui))
            .style(ui.styles.root)
            .block(
                Block::new()
                    .borders(Borders::TOP)
                    .style(ui.styles.accent)
                    .title("english 50".bold())
                    .padding(Padding::horizontal(1)),
            )
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }

    /// Convert all testwords to styled spans with spacing, returned as a single line
    pub fn words_to_line(&self, ui: &Ui) -> Line<'_> {
        // TODO this is slow but idfc
        let mut spans: Vec<Span> = Vec::with_capacity(self.words.len() * (2.2 as usize));
        for (wc, word) in self.words.iter().enumerate() {
            let mut i = 0;
            let mut len = word.word.len();
            for ev in word.presses.iter() {
                // backspace
                if ev.key == BKSPC {
                    if i > 0 {
                        let _ = spans.pop();
                        i -= 1;
                    }
                }
                // potential correct press
                else if i < len {
                    if ev.key == word.word.chars().nth(i).unwrap() {
                        spans.push(Span::raw(ev.key.to_string()).style(ui.styles.typed));
                    } else {
                        spans.push(Span::raw(ev.key.to_string()).style(ui.styles.incorrect));
                    }
                    i += 1;
                }
                // incorrect press
                else {
                    spans.push(Span::raw(ev.key.to_string()).style(ui.styles.incorrect));
                    i += 1;
                }
            }
            // rest of text is untyped, if any
            if wc == self.word_i
                && let Some(c) = word.word.chars().nth(i)
            {
                spans.push(Span::raw(c.to_string()).style(ui.styles.accent));
                i += 1;
            }
            spans.push(Span::raw(&word.word[min(i, len)..]).style(ui.styles.untyped));
            spans.push(Span::raw(" "));
        }
        let _ = spans.pop(); // remove trailing space
        return Line::from(spans);
    }
}
