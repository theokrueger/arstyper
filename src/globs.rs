//! Global state module.
//!
//! Exposes thread-safe access to read-only configuration and styles,
//! as well as the mutable ScoreManager.

use crate::config::Config;
use crate::scoremanager::ScoreManager;
use ratatui::style::Style;
use std::sync::{Mutex, MutexGuard, OnceLock};

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
    pub dark_text: Style,
}

pub struct Globs {
    pub scoremgr: Mutex<ScoreManager>,
    pub cfg: Config,
    pub sty: Styles,
}

pub static GLOBS: OnceLock<Globs> = OnceLock::new();

/// Retrieve a reference to the global configuration.
pub fn cfg() -> &'static Config {
    &GLOBS.get().expect("GLOBS not initialized").cfg
}

/// Retrieve a reference to the global styles.
pub fn sty() -> &'static Styles {
    &GLOBS.get().expect("GLOBS not initialized").sty
}

/// Lock and retrieve the mutable score manager.
pub fn scoremgr() -> MutexGuard<'static, ScoreManager> {
    GLOBS.get().unwrap().scoremgr.lock().unwrap()
}
