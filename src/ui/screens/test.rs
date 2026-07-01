//! Typing test struct
use crate::{
    sty,
    traits::{ArstyperWidget, ArstyperWidgetState},
    ui::{Screen, UiRequest},
};

use ratatui::{
    buffer::Buffer,
    crossterm::event::{KeyCode, KeyEvent, KeyModifiers},
    layout::Rect,
    style::Stylize,
    text::{Line, Span},
    widgets::{Block, Borders, Padding, Paragraph, StatefulWidgetRef, Widget, Wrap},
};
use std::{io, sync::mpsc::SyncSender, time::Instant};

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
struct TestWord<'a> {
    word: String,
    /// Series of keypressed used to build `spans`
    presses: Vec<Keypress>,
    /// Spans that compose this typed word
    spans: Vec<Span<'a>>,
}

impl TestWord<'_> {
    /// Styled spans showing typed, untyped, cursor, etc
    fn update_span_vec(&mut self, show_cursor: bool) {
        let mut state = true; // true for correct
        let mut typed = String::new();
        self.spans.clear();

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
                    self.spans.push(Span::styled(typed, sty!(incorrect)));
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
                    self.spans.push(Span::styled(typed, sty!(typed)));
                    // flip state
                    state = !state;
                    typed = String::new();
                    // add incorrect
                    typed.push(p);
                }
            }
        }

        self.spans.push(Span::styled(
            typed,
            if state { sty!(typed) } else { sty!(incorrect) },
        ));

        // fill remaining word
        if let Some(c) = t_i.next() {
            let s = c.to_string();
            self.spans.push(Span::styled(
                s,
                if show_cursor {
                    sty!(cursor)
                } else {
                    sty!(untyped)
                },
            ));

            typed = t_i.collect();
            typed.push(' ');
            self.spans.push(Span::styled(typed, sty!(untyped)));
        } else {
            self.spans.push(Span::styled(
                " ",
                if show_cursor {
                    sty!(cursor)
                } else {
                    sty!(untyped)
                },
            ));
        }
    }
    fn from(string: String) -> Self {
        let mut tw = Self {
            presses: Vec::with_capacity(string.len()),
            word: string,
            spans: Vec::new(),
        };

        tw.update_span_vec(false);
        return tw;
    }
}

impl TestWord<'_> {
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
pub struct TestState<'a> {
    words: Vec<TestWord<'a>>,
    word_i: usize,
    title: String,
}

impl ArstyperWidgetState for TestState<'_> {
    fn new() -> io::Result<Self> {
        Ok(Self {
            words: Vec::new(),
            word_i: 0,
            title: "".to_string(),
        })
    }
}

pub struct Test {
    /// Message to the UI to be performed on next tick. Didn't feel like using an actual message system lmao
    tx: SyncSender<UiRequest>,
}

impl ArstyperWidget for Test {
    fn new(tx: SyncSender<UiRequest>) -> io::Result<Self> {
        Ok(Self { tx: tx })
    }

    fn handle_events(&mut self, key: KeyEvent, state: &mut TestState) {
        match key.code {
            KeyCode::Char(' ') => {
                // remove cursor from current
                state.words[state.word_i]
                    .presses
                    .push(Keypress::from_chr(' '));
                state.words[state.word_i].update_span_vec(false);
                // increment to next when applicable
                state.word_i += 1;
            }
            KeyCode::Char(chr) => {
                state.words[state.word_i]
                    .presses
                    .push(Keypress::from_chr(chr));
            }
            KeyCode::Tab => self
                .tx
                .send(UiRequest::ChangeScreen(Screen::ResultsScreen))
                .unwrap(),
            KeyCode::Backspace => {
                // should go to previous word?
                if state.words[state.word_i].presses.len() == 0 {
                    if state.word_i != 0 {
                        // remove cursor from current
                        state.words[state.word_i].update_span_vec(false);
                        // decrement to previous
                        state.word_i -= 1;
                    }
                }
                // (ctrl|alt) + backspace -> delete entire word
                if key
                    .modifiers
                    .iter()
                    .any(|m| m == KeyModifiers::CONTROL || m == KeyModifiers::ALT)
                {
                    state.words[state.word_i].presses.clear();
                }
                // just backspace
                else {
                    state.words[state.word_i].presses.pop();
                }
            }
            _ => {}
        }
        // check for completion
        if state.word_i >= state.words.len() - 1 && state.words[state.words.len() - 1].is_typed() {
            self.tx
                .send(UiRequest::ChangeScreen(Screen::ResultsScreen))
                .unwrap();
        } else {
            // update display
            state.words[state.word_i].update_span_vec(true);
        }
    }
}

impl StatefulWidgetRef for Test {
    type State = TestState<'static>;
    fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let _word_i = state.word_i;
        Paragraph::new(Line::from(
            state
                .words
                .iter()
                .map(|tw| tw.spans.clone())
                .flatten()
                .collect::<Vec<Span>>(),
        ))
        .style(sty!(root))
        .block(
            Block::new()
                .borders(Borders::TOP)
                .style(sty!(accent))
                .title(state.title.as_str().bold())
                .padding(Padding::horizontal(1)),
        )
        .wrap(Wrap { trim: true })
        .render(area, buf);
    }
}

impl TestState<'_> {
    /// Set title
    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    /// Create test from an iterator over string items
    pub fn test_from(&mut self, words: impl Iterator<Item = String>) {
        self.words = words
            .map(|w| TestWord::from(w.to_lowercase()))
            .collect::<Vec<TestWord>>();
        self.word_i = 0;
        self.words[0].update_span_vec(true);
    }
}
