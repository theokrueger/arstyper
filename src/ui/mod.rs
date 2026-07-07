//! Root UI
pub mod color_preview;
pub mod overlays;
pub mod screens;

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

#[derive(PartialEq, Clone)]
/// Currenly enabled overlay
pub enum Overlay {
    Menu,
}

#[derive(Clone)]
/// Request sent by screens to here
pub enum UiRequest {
    /// Do nothing
    Empty,
    /// Exit the program
    Exit,
    /// Change the screen (duh)
    ChangeScreen(Screen),
    /// Go to the last screen
    GoToLastScreen,
    /// Clear status
    ClearStatus,
    /// Set the overlay
    AddOverlay(Overlay),
    /// Remove the current overlay
    RemoveOverlay,
    /// Discard current test and create a new one using current settings
    NewTest,
    /// Update score in Results from ScoreManager
    UpdateResults,
    //// Set the statusbar to this message. Will overwrite any existing message
    //DisplayStatus(String, DateTime<Local>),
}
