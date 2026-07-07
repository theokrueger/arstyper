//! arstyper
mod config;
mod globs;
mod lang;
mod scoremanager;
mod traits;
mod ui;
mod util;

use config::Config;
use globs::{Globs, Styles};
use scoremanager::ScoreManager;
use ui::{
    AppState,
    screens::main::{Main, MainState},
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
use std::{cell::Cell, io::stdout, sync::Mutex};

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
    let cfg = Config::get().unwrap_or_else(err_disp!("Config"));
    let root_sty = Style::new().fg(cfg.theme.fg).bg(cfg.theme.bg);
    let accent_sty = root_sty.fg(cfg.theme.accent);
    let modeline_sty = root_sty.bg(cfg.theme.accent);
    let sty = Styles {
        root: root_sty,
        root_inv: root_sty.add_modifier(Modifier::REVERSED),
        modeline_inv: modeline_sty.add_modifier(Modifier::REVERSED),
        modeline: modeline_sty,
        accent_inv: accent_sty.add_modifier(Modifier::REVERSED),
        accent: accent_sty,
        untyped: root_sty.fg(cfg.theme.untyped_text),
        typed: root_sty.fg(cfg.theme.typed_text),
        incorrect: root_sty.fg(cfg.theme.incorrect_text),
        cursor: root_sty.bg(cfg.theme.accent),
    };
    let scoremgr = Mutex::new(ScoreManager::new(&cfg)?);

    unsafe {
        globs::GLOBS
            .set(Globs { scoremgr, cfg, sty })
            .unwrap_unchecked();
    }

    // init ui
    let mut ui_state = MainState::new().unwrap_or_else(err_disp!("UI Setup"));
    let ui = Main::new();

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
