//! due to unsafe blocks, it is assumed that every OnceCell herein is initialised before UI init.
//! therefore, it will always be memory safe even with unchecked unwraps

use crate::config::Config;
use crate::scoremanager::ScoreManager;
use once_cell::sync::OnceCell;
use ratatui::style::Style;
use std::{ops::DerefMut, sync::Mutex};

pub struct Globs {
    // mutable
    pub scoremgr: Mutex<ScoreManager>,

    // immutable
    pub cfg: Config,
    pub sty: Styles,
}

pub static GLOBS: OnceCell<Globs> = OnceCell::new();

/// Apply function to a mutex inside glob state
#[macro_export]
macro_rules! globs_apply {
    ($name:ident, $func:expr) => {
        // TODO make this not stupid in our syncrhonous context
        use std::ops::DerefMut;
        unsafe {
            let mut res = crate::globs::GLOBS
                .get()
                .unwrap_unchecked()
                .$name
                .lock()
                .unwrap_unchecked();
            let field = res.deref_mut();
            $func(field);
        }
    };
}

/// Get an immutable glob by ident[.field.field]
#[macro_export]
macro_rules! globs {
    ($name:ident$(.$field:ident)*) => {
        unsafe { crate::globs::GLOBS.get().unwrap_unchecked().$name$(.$field)* }
    };
}

/// Get immutable glob by reference
#[macro_export]
macro_rules! globs_ref {
    ($name:ident$(.$field:ident)*) => {
        unsafe { &crate::globs::GLOBS.get().unwrap_unchecked().$name$(.$field)* }
    };
}

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

// handy due to frequent use
#[macro_export]
macro_rules! config {
    ($name:ident$(.$field:ident)*) => {
        crate::globs!(cfg.$name$(.$field)*)
    };
}

#[macro_export]
macro_rules! sty {
    ($name:ident) => {
        crate::globs!(sty.$name)
    };
}
