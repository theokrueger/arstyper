//! arstyper
mod config;
mod lang;
mod traits;
mod ui;

use config::Config;
use lang::Lang;
use traits::ArstyperScreen;
use ui::{AppState, Overlay, Ui, UiRequest, UiState};

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
    widgets::{Paragraph, StatefulWidget, StatefulWidgetRef, Widget, Wrap},
};
use std::{
    io::stdout,
    rc::Rc,
    sync::mpsc::{Receiver, SyncSender, sync_channel},
};

macro_rules! err_disp {
    ($name:literal) => {
        |e| {
            println!("Fatal Error in {}: {}", $name, e);
            std::process::exit(1);
        }
    };
}

fn main() -> std::io::Result<()> {
    let cfg = Config::get().unwrap_or_else(err_disp!("Config"));
    let mut ui_state = UiState::new(cfg).unwrap_or_else(err_disp!("UI Setup"));
    let ui = Ui::new();

    let mut terminal = ratatui::init();

    // enter raw mode
    let mut stdout = stdout();
    execute!(
        stdout,
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES)
    )?;

    while ui_state.state != AppState::Stopped {
        terminal.draw(|frame| frame.render_stateful_widget(&ui, frame.area(), &mut ui_state))?;

        ui.tick(&mut ui_state).unwrap_or_else(err_disp!("UI Tick"));
    }

    // exit raw mode
    execute!(stdout, PopKeyboardEnhancementFlags)?;
    ratatui::restore();

    Ok(())
}
