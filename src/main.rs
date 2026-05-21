//! arstyper
mod config;
mod lang;
mod traits;
mod ui;

use config::Config;
use traits::ArstyperScreen;
use ui::{
    AppState,
    ui::{Ui, UiState},
};

use ratatui::crossterm::{
    event::{KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute,
};
use std::io::stdout;

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
