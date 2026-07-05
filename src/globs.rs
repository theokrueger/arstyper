//! due to unsafe blocks, it is assumed that every OnceCell herein is initialised before UI init.
//! therefore, it will always be memory safe even with unchecked unwraps

use crate::config::Config;
use crate::scoremanager::ScoreManager;
use once_cell::sync::OnceCell;
use ratatui::style::Style;

pub struct Globs {
    scoremgr: ScoreManager,
    cfg: Config,
    sty: Styles,
}

pub static GLOBS: OnceCell<Globs> = OnceCell::new();

/* SCOREMANAGER */
pub static SCOREMANAGER: OnceCell<ScoreManager> = OnceCell::new();

#[macro_export]
macro_rules! scoremanager {
    () => {
        unsafe { crate::globs::SCOREMANAGER.get().unwrap_unchecked() }
    };
}

/* CONFIG */
pub static CONFIG: OnceCell<Config> = OnceCell::new();

#[macro_export]
macro_rules! config {
    ($name:ident$(.$field:ident)*) => {
        unsafe { crate::globs::CONFIG.get().unwrap_unchecked().$name$(.$field)* }
    };
}

#[macro_export]
macro_rules! config_ref {
    ($name:ident$(.$field:ident)*) => {
        unsafe { &crate::globs::CONFIG.get().unwrap_unchecked().$name$(.$field)* }
    };
}

/* UI STYLE */
pub static STYLES: OnceCell<Styles> = OnceCell::new();

/// Common style shortcuts
pub struct Styles {
    pub root: Style,
    pub root_inv: Style,
    pub modeline: Style,
    pub modeline_inv: Style,
    pub accent: Style,
    pub accent_inv: Style,
    pub untyped: Style,
    pub typed: Style,
    pub incorrect: Style,
    pub cursor: Style,
}

#[macro_export]
macro_rules! sty {
    ($name:ident) => {
        unsafe { crate::globs::STYLES.get().unwrap_unchecked().$name }
    };
}
