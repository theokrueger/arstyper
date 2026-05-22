//! Typing test struct
use crate::{
    traits::{ArstyperWidget, ArstyperWidgetState},
    ui::{Screen, Styles, UiRequest},
};

use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::Rect,
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, StatefulWidgetRef, Widget, Wrap},
};
use std::{io, rc::Rc, sync::mpsc::SyncSender, time::Instant};

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
    /// Series of keypressed used to build `spans`
    presses: Vec<Keypress>,
}

impl TestWord {
    /// Styled spans showing typed, untyped, cursor, etc
    fn as_span_vec(
        &self,
        show_cursor: bool,
        correct: Style,
        incorrect: Style,
        untyped: Style,
        cursor: Style,
    ) -> Vec<Span<'_>> {
        let mut state = true; // true for correct
        let mut typed = String::new();
        let mut spans = Vec::<Span>::new();

        let mut t_i = self.word.chars();
        let mut p_i = self.presses.iter().map(|x| x.key);

        // check presses
        while let Some(p) = p_i.next()
            && p != ' '
        {
            if let Some(t) = t_i.next()
                && p == t
            {
                // typed correctly
                if state {
                    // prev was correct as well
                    typed.push(p);
                } else {
                    // push incorrects
                    spans.push(Span::styled(typed, incorrect));
                    // flip state
                    state = !state;
                    typed = String::new();
                    // add correct
                    typed.push(p);
                }
            } else {
                // typed incorrectly
                if !state {
                    // prev was incorrect as well
                    typed.push(p);
                } else {
                    // push corrects
                    spans.push(Span::styled(typed, correct));
                    // flip state
                    state = !state;
                    typed = String::new();
                    // add incorrect
                    typed.push(p);
                }
            }
        }

        spans.push(Span::styled(typed, if state { correct } else { incorrect }));

        // fill remaining word
        if let Some(c) = t_i.next() {
            let s = c.to_string();
            spans.push(Span::styled(s, if show_cursor { cursor } else { untyped }));

            typed = t_i.collect();
            typed.push(' ');
            spans.push(Span::styled(typed, untyped));
        } else {
            spans.push(Span::styled(
                " ",
                if show_cursor { cursor } else { untyped },
            ));
        }
        return spans;
    }
}

impl From<String> for TestWord {
    fn from(string: String) -> Self {
        Self {
            presses: Vec::with_capacity(string.len()),
            word: string,
        }
    }
}

impl TestWord {
    /// Is the word fully and correctly typed
    fn is_correct(&self) -> bool {
        return self
            .presses
            .iter()
            .filter_map(|x| if x.key == ' ' { None } else { Some(x.key) })
            .collect::<String>()
            == self.word;
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

/// The actual typing test info
pub struct TestState {
    words: Vec<TestWord>,
    word_i: usize,
    title: String,
}

impl ArstyperWidgetState for TestState {
    fn new() -> io::Result<Self> {
        Ok(Self {
            words: Vec::new(),
            word_i: 0,
            title: "".to_string(),
        })
    }
}

pub struct Test {
    styles: Rc<Styles>,
    /// Message to the UI to be performed on next tick. Didn't feel like using an actual message system lmao
    tx: SyncSender<UiRequest>,
}

impl ArstyperWidget for Test {
    fn new(s: Rc<Styles>, tx: SyncSender<UiRequest>) -> io::Result<Self> {
        Ok(Self { styles: s, tx: tx })
    }

    fn handle_events(&mut self, key: KeyEvent, state: &mut TestState) {
        let mut word = &mut state.words[state.word_i];
        match key.code {
            KeyCode::Char(' ') => {
                word.presses.push(Keypress::from_chr(' '));
                state.word_i += 1;
            }
            KeyCode::Char(chr) => {
                word.presses.push(Keypress::from_chr(chr));
            }
            KeyCode::Tab => self
                .tx
                .send(UiRequest::ChangeScreen(Screen::ResultsScreen))
                .unwrap(),
            KeyCode::Backspace => {
                // should go to previous word?
                if word.presses.len() == 0 {
                    if state.word_i != 0 {
                        state.word_i -= 1;
                        word = &mut state.words[state.word_i];
                    }
                }
                // (ctrl|alt) + backspace -> delete entire word
                if key
                    .modifiers
                    .iter()
                    .any(|m| m == KeyModifiers::CONTROL || m == KeyModifiers::ALT)
                {
                    word.presses.clear();
                }
                // just backspace
                else {
                    word.presses.pop();
                }
            }
            _ => {}
        }
        // check for completion
        if state.word_i >= state.words.len() - 1 && state.words[state.words.len() - 1].is_typed() {
            self.tx
                .send(UiRequest::ChangeScreen(Screen::ResultsScreen))
                .unwrap();
        }
    }
}

impl StatefulWidgetRef for Test {
    type State = TestState;
    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let word_i = state.word_i;
        Paragraph::new(Line::from(
            state
                .words
                .iter()
                .enumerate()
                .map(|(i, tw)| {
                    tw.as_span_vec(
                        word_i == i,
                        self.styles.typed,
                        self.styles.incorrect,
                        self.styles.untyped,
                        self.styles.cursor,
                    )
                })
                .flatten()
                .collect::<Vec<Span>>(),
        ))
        .style(self.styles.root)
        .block(
            Block::new()
                .borders(Borders::TOP)
                .style(self.styles.accent)
                .title(state.title.as_str().bold())
                .padding(Padding::horizontal(1)),
        )
        .wrap(Wrap { trim: true })
        .render(area, buf);
    }
}

impl TestState {
    /// Set title
    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    /// Create test from an iterator over string items
    pub fn test_from(&mut self, words: impl Iterator<Item = String>) {
        self.words = words
            .map(|w| w.to_lowercase().into())
            .collect::<Vec<TestWord>>();
        self.word_i = 0;
    }
}
