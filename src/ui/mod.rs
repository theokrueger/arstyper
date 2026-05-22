//! Root UI
mod about;
pub mod color_preview;
mod menubar;
mod results;
mod stats;
mod test;
pub mod ui;

use ratatui::style::Style;
use strum::{Display, EnumIter, FromRepr};

#[derive(Default, PartialEq)]
pub enum AppState {
    #[default]
    Running,
    Stopped,
}

#[derive(Default, Display, Clone, FromRepr, EnumIter, PartialEq)]
/// Screen to display in body area
pub enum Screen {
    #[default]
    #[strum(to_string = "Testing")]
    TestScreen,
    #[strum(to_string = "Results")]
    ResultsScreen,
    #[strum(to_string = "Stats")]
    StatsScreen,
    #[strum(to_string = "About")]
    AboutScreen,
}

#[derive(Default, PartialEq, Clone)]
/// Currenly enabled overlay
pub enum Overlay {
    #[default]
    None,
    MenuBar,
}

#[derive(Clone)]
/// Request sent by screens to here
pub enum UiRequest {
    /// Exit the program
    Exit,
    /// Change the screen (duh)
    ChangeScreen(Screen),
    /// Go to the last screen
    GoToLastScreen,
    /// Clear status
    ClearStatus,
    /// Set the overlay
    ShowOverlay(Overlay),
    /// Discard current test and create a new one using current settings
    NewTest,
    //// Set the statusbar to this message. Will overwrite any existing message
    //DisplayStatus(String, DateTime<Local>),
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
