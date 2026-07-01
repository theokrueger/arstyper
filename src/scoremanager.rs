//! Create and manage new scores, score database, and perform analyses.
use crate::config;
use crate::config::Config;

use chrono::{DateTime, Local, TimeDelta, Timelike, Utc};
use std::{
    fs::{self, File},
    io::{self, BufRead, Error, ErrorKind},
    path::PathBuf,
    time::{Duration, Instant},
};

/// A single keypress
pub struct Keypress {
    pub key: char,
    pub time: Instant,
}

impl Keypress {
    /// Create keypress from char with current time as instant
    pub fn from_chr(key: char) -> Self {
        Self {
            key: key,
            time: Instant::now(),
        }
    }
}

/// A score word as series of presses
pub struct ScoreWord {
    word: String,
    presses: Vec<Keypress>,
}

//#[derive(Serialize, Deserialize)]
/// Test statistics derived from raw data
pub struct Score {
    completed: DateTime<Utc>,
    chars: u32,
    correct_strokes: u32,
    incorrect_strokes: u32,
    duration: Duration,
}

impl Score {
    /// WPM or CPM string depending on config
    pub fn speed_string(&self, raw: bool) -> String {
        let cpm = if raw {
            (self.correct_strokes + self.incorrect_strokes) as f32
        } else {
            self.correct_strokes as f32
        } / self.duration.as_secs_f32();
        if config!(locale.cpm_over_wpm) {
            format!("{cpm} CPM")
        } else {
            format!("{} WPM", cpm / 5.0)
        }
    }
}

pub struct ScoreManager {}

impl ScoreManager {
    pub fn new(cfg: &Config) -> Self {
        Self {}
    }

    fn path() -> PathBuf {
        dirs::data_local_dir().unwrap().join("arstyper/scores")
    }

    pub fn save(score: Score) -> Result<(), Error> {
        let p = &Self::path().join(score.completed.to_string());
        // let f = File::create(&p).or(Err(Error::new(
        //     ErrorKind::NotFound,
        //     format!("Unable to create file '{p}'"),
        // )))?;
        // let buf
        Ok(())
    }
}
