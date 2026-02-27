//! Typing test struct
use crate::ui::Styles;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::Rect,
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, Widget, Wrap},
};
use std::{cmp::min, time::Instant};

pub const BKSPC: char = 0x08 as char;
pub const WORD_BKSPC: char = 0x18 as char;

/// A single keypress
struct Keypress {
    _key: char,
    _time: Instant,
}

impl Keypress {
    /// Create keypress from char with current time as instant
    fn from_chr(key: char) -> Self {
        Self {
            _key: key,
            _time: Instant::now(),
        }
    }
}

/// A single test word and its keypresses.
struct TestWord<'a> {
    word: String,
    presses: Vec<Keypress>,
    spans: Vec<Span<'a>>,
}

impl From<String> for TestWord<'_> {
    fn from(string: String) -> Self {
        TestWord {
            presses: Vec::with_capacity(string.len()),
            word: string,
            spans: Vec::new(),
        }
    }
}

/// The actual typing test
pub struct Test<'a> {
    words: Vec<TestWord<'a>>,
    word_i: usize,
    styles: Styles,
}

impl<'a> Test<'a> {
    /// Create a new emtpy test, which must be initialised before use :D
    pub fn new(s: Styles) -> Self {
        Test {
            words: Vec::new(),
            word_i: 0,
            styles: s,
        }
    }

    /// Handle keypress events for this test
    pub fn handle_events(&mut self, key: KeyEvent) {
        let mut word = &mut self.words[self.word_i];
        match key.code {
            KeyCode::Char(' ') => {
                self.word_i += 1;
            }
            KeyCode::Char(chr) => {
                word.presses.push(Keypress::from_chr(chr));

                let len = word.spans.len();
                // potential correct press
                if len < word.word.len() && chr == word.word.chars().nth(len).unwrap() {
                    word.spans
                        .push(Span::raw(chr.to_string()).style(self.styles.typed));
                }
                // incorrect press
                else {
                    word.spans
                        .push(Span::raw(chr.to_string()).style(self.styles.incorrect));
                }
            }
            KeyCode::Backspace => {
                // (ctrl|alt) + backspace -> delete entire word
                if key
                    .modifiers
                    .iter()
                    .any(|m| m == KeyModifiers::CONTROL || m == KeyModifiers::ALT)
                {
                    // delete last word cause nothing was typed for this one
                    if word.spans.len() == 0 {
                        self.word_i -= 1;
                        word = &mut self.words[self.word_i];
                    }

                    word.presses.push(Keypress::from_chr(WORD_BKSPC));
                    word.spans = Vec::new();
                }
                // just backspace
                else {
                    word.presses.push(Keypress::from_chr(BKSPC));
                    let _ = word.spans.pop();
                    if self.word_i > 0 && word.spans.len() == 0 {
                        self.word_i -= 1;
                    }
                }
            }
            _ => {}
        }
    }

    /// Return full word as vec of spans, including untyped portion
    fn tw_as_span_vec(&self, word_i: usize, tw: &TestWord<'a>) -> Vec<Span<'a>> {
        // typed portion
        let mut sv = tw.spans.clone();

        // cursor
        if self.word_i == word_i {
            match tw.word.chars().nth(sv.len()) {
                Some(c) => sv.push(Span::raw(c.to_string()).style(self.styles.cursor)),
                None => {
                    // must be end of string, add stylized space and return.
                    sv.push(Span::raw(' '.to_string()).style(self.styles.cursor));
                    return sv;
                }
            };
        }

        // untyped portion
        let idx = min(sv.len(), tw.word.len());
        let ut = tw.word[idx..].to_string() + " ";
        sv.push(Span::raw(ut).style(self.styles.untyped));
        return sv;
    }

    /// Create test from an iterator over string items
    pub fn test_from(&mut self, words: impl Iterator<Item = String>) {
        self.words = words
            .map(|w| w.to_lowercase().into())
            .collect::<Vec<TestWord>>();
    }

    /// Render the test text
    pub fn render(&self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.words_to_line())
            .style(self.styles.root)
            .block(
                Block::new()
                    .borders(Borders::TOP)
                    .style(self.styles.accent)
                    .title("english 50".bold())
                    .padding(Padding::horizontal(1)),
            )
            .wrap(Wrap { trim: true })
            .render(area, buf);
    }

    /// Convert all testwords to styled spans with spacing, returned as a single line
    pub fn words_to_line(&self) -> Line<'a> {
        Line::from(
            self.words
                .iter()
                .enumerate()
                .map(|(i, tw)| self.tw_as_span_vec(i, tw))
                .flatten()
                .collect::<Vec<Span>>(),
        )
    }
}
