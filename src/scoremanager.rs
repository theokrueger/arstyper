//! Create and manage new scores, score database, and perform analyses.
use crate::{config, config::Config, util};

use chrono::{DateTime, Local, TimeDelta, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{self, BufRead, Error, ErrorKind},
    path::PathBuf,
    time::{Duration, Instant, SystemTime},
};

/// A single keypress
pub struct Keypress {
    pub key: char,
    pub time: Instant,
    pub correct: bool,
    pub ignore: bool,
}

impl Keypress {
    /// Create keypress from char with current time as instant
    pub fn from_chr(key: char, correct: bool) -> Self {
        Self {
            key: key,
            time: Instant::now(),
            ignore: false,
            correct,
        }
    }
}

/// A score word as series of presses
pub struct ScoreWord {
    pub word: String,
    pub presses: Vec<Keypress>,
}

impl From<String> for ScoreWord {
    fn from(string: String) -> Self {
        Self {
            word: string,
            presses: Vec::new(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
/// Test statistics derived from raw data
pub struct Score {
    words: u32,
    completed: u64,
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

impl From<Vec<ScoreWord>> for Score {
    /// Completed field is at time of conversion
    fn from(sws: Vec<ScoreWord>) -> Self {
        let mut correct_strokes = 0;
        let mut incorrect_strokes = 0;
        let mut chars = 0;

        // count stats
        for sw in &sws {
            chars += sw.word.len() + 1; // +1 for space
            for kp in sw.presses.iter() {
                if kp.correct {
                    correct_strokes += 1;
                } else {
                    incorrect_strokes += 1;
                }
            }
        }

        // head/tail operations
        let mut duration = unsafe {
            let start = sws
                .first()
                .unwrap_unchecked()
                .presses
                .first()
                .unwrap_unchecked()
                .time;
            let end = sws
                .last()
                .unwrap_unchecked()
                .presses
                .last()
                .unwrap_unchecked()
                .time;
            end - start
        };

        // correct the number of chars depending on if the user typed a space
        unsafe {
            if sws
                .last()
                .unwrap_unchecked()
                .presses
                .last()
                .unwrap_unchecked()
                .key
                != ' '
            {
                chars -= 1;
            }
        }

        Self {
            words: sws.len() as u32,
            chars: chars as u32,
            correct_strokes,
            incorrect_strokes,
            completed: util::timestamp_s(),
            duration,
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

    pub fn save_score(&mut self, score: Score) -> Result<(), Error> {
        let p = &Self::path().join(format!("{}.json", score.completed.to_string()));
        let f = File::create(&p).or(Err(Error::new(
            ErrorKind::NotFound,
            format!("Unable to create file '{}'", p.display()),
        )))?;
        println!("{:?}", score);
        Ok(())
    }
}
