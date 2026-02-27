//! Typing test struct
use crate::ui::{Screen, Styles, UiRequest};

use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::Rect,
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, Widget, Wrap},
};
use std::{cmp::min, sync::mpsc::SyncSender, time::Instant};

/// A normal backspace
pub const BKSPC: char = 0x08 as char;
/// A "backspace" for deleting an entire word
pub const WORD_BKSPC: char = 0x18 as char;

/// A single keypress
struct Keypress {
    key: char,
    _time: Instant,
}

impl Keypress {
    /// Create keypress from char with current time as instant
    fn from_chr(key: char) -> Self {
        Self {
            key: key,
            _time: Instant::now(),
        }
    }
}

/// A single test word and its keypresses.
struct TestWord<'a> {
    word: String,
    presses: Vec<Keypress>,
    /// Renderable, incremental text "object"
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

impl TestWord<'_> {
    /// Is the word fully and correctly typed
    fn is_correct(&self) -> bool {
        let mut s: String = "".to_string();
        for e in self.presses.iter() {
            match e.key {
                ' ' => (),
                BKSPC => {
                    s.pop();
                }
                WORD_BKSPC => s = "".to_string(),
                _ => s.push(e.key),
            }
        }
        return s == self.word;
    }

    /// Does the word end in a space (has been typed, incorrectly or correctly)
    fn is_typed(&self) -> bool {
        if let Some(lp) = self.presses.last()
            && lp.key == ' '
        {
            true
        } else {
            self.is_correct()
        }
    }
}

/// The actual typing test
pub struct Test<'a> {
    words: Vec<TestWord<'a>>,
    word_i: usize,
    styles: Styles,
    /// Message to the UI to be performed on next tick. Didn't feel like using an actual message system lmao
    tx: SyncSender<UiRequest>,
}

impl<'a> Test<'a> {
    /// Create a new emtpy test, which must be initialised before use :D
    pub fn new(s: Styles, tx: SyncSender<UiRequest>) -> Self {
        Test {
            words: Vec::new(),
            word_i: 0,
            styles: s,
            tx: tx,
        }
    }

    /// Handle keypress events for this test
    pub fn handle_events(&mut self, key: KeyEvent) {
        let mut word = &mut self.words[self.word_i];
        match key.code {
            KeyCode::Char(' ') => {
                word.presses.push(Keypress::from_chr(' '));
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
        // check for completion
        if self.word_i >= self.words.len() - 1 && self.words[self.words.len() - 1].is_typed() {
            self.tx
                .send(UiRequest::ChangeScreen(Screen::ResultsScreen))
                .unwrap();
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

    /// Convert all testwords to styled spans with spacing, returned as a single line so that it wraps properly
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keypress_correct() {
        let tests: Vec<(&str, Vec<char>, bool)> = vec![
            ("test", vec!['t', 'e', 's', 't'], true),
            ("test", vec![' ', 'q', BKSPC, 't', 'e', 's', 't', ' '], true),
            (
                "test",
                vec![
                    't', 'e', 's', 't', 'a', 'b', 'c', WORD_BKSPC, 't', 'e', 's', 't', ' ',
                ],
                true,
            ),
            ("abcd", vec!['a', 'b', 'c', 'd', 'e'], false),
        ];
        for (word, chars, correct) in tests.into_iter() {
            let mut tw: TestWord = word.to_string().into();
            tw.presses = chars.into_iter().map(|c| Keypress::from_chr(c)).collect();
            assert_eq!(tw.is_correct(), correct)
        }
    }
}
