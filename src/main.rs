//! arstyper
mod config;
mod consts;
mod lang;
mod traits;
mod ui;

use config::Config;
use consts::Styles;
use traits::ArstyperScreen;
use ui::{
    AppState,
    ui::{Ui, UiState},
};

use ratatui::{
    crossterm::{
        event::{
            KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
        },
        execute,
    },
    style::{Modifier, Style},
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
    // global 'consts' init
    unsafe {
        consts::CONFIG
            .set(Config::get().unwrap_or_else(err_disp!("Config")))
            .unwrap_unchecked();
        let root_sty = Style::new().fg(config!(theme.fg)).bg(config!(theme.bg));
        consts::STYLES
            .set(Styles {
                root: root_sty,
                root_inv: root_sty.add_modifier(Modifier::REVERSED),
                modeline: root_sty.bg(config!(theme.accent)),
                modeline_inv: root_sty
                    .bg(config!(theme.accent))
                    .add_modifier(Modifier::REVERSED),
                accent: root_sty.fg(config!(theme.accent)),
                accent_inv: root_sty
                    .fg(config!(theme.accent))
                    .add_modifier(Modifier::REVERSED),
                untyped: root_sty.fg(config!(theme.untyped_text)),
                typed: root_sty.fg(config!(theme.typed_text)),
                incorrect: root_sty.fg(config!(theme.incorrect_text)),
                cursor: root_sty.bg(config!(theme.accent)),
            })
            .unwrap_unchecked();
    }

    // init ui
    let mut ui_state = UiState::new().unwrap_or_else(err_disp!("UI Setup"));
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
