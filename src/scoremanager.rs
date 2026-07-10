//! Create and manage new scores, score database, and perform analyses.
use crate::{config::Config, globs, util};

use serde::{Deserialize, Serialize};
use std::{
    fs::{self, File},
    io::{Error, ErrorKind, Write},
    path::PathBuf,
    time::{Duration, Instant},
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
            key,
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

#[derive(Deserialize, Serialize, Default)]
/// Test statistics derived from raw data
pub struct Score {
    pub lang: String,
    pub words: u32,
    pub completed: u64,
    pub chars: u32,
    pub correct_strokes: u32,
    pub incorrect_strokes: u32,
    pub duration: Duration,
}

impl Score {
    /// WPM or CPM string depending on config
    pub fn speed(&self, raw: bool) -> f32 {
        let mut cpm = (if raw {
            (self.correct_strokes + self.incorrect_strokes) as f32
        } else {
            self.correct_strokes as f32
        }) / self.duration.as_secs_f32()
            * 60.0;
        if !globs::cfg().locale.cpm_over_wpm {
            cpm /= 6.0;
        }
        cpm
    }

    /// Overall Accuracy as % of keystrokes that were correct
    pub fn accuracy(&self) -> f32 {
        100.0 * self.correct_strokes as f32 / (self.correct_strokes + self.incorrect_strokes) as f32
    }

    /// Completion % of test
    pub fn completion(&self) -> f32 {
        100.0 * self.correct_strokes as f32 / self.chars as f32
    }

    /// Check if score is valid
    pub fn valid(&self) -> bool {
        self.accuracy() > 70.0 && self.completion() > 99.9
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
        let mut duration = Duration::from_secs(0);
        if let Some(first) = sws.first().unwrap().presses.first()
        // ignore doesnt matter, test starts on first keystroke no matter what
        {
            'outer: for sw in sws.iter().rev() {
                for kp in sw.presses.iter().rev() {
                    if !kp.ignore {
                        duration = kp.time - first.time;
                        break 'outer;
                    }
                }
            }
        }

        // correct the number of chars depending on if the user typed a space
        if let Some(sw) = sws.last()
            && let Some(kp) = sw.presses.last()
            && kp.key != ' '
        {
            chars -= 1;
        }

        Self {
            lang: globs::cfg().lang.clone(),
            words: sws.len() as u32,
            chars: chars as u32,
            correct_strokes,
            incorrect_strokes,
            completed: util::timestamp_s(),
            duration,
        }
    }
}

pub struct ScoreManager {
    pub score: Score,
    pub tests_taken: usize,
}

impl ScoreManager {
    pub fn new(_cfg: &Config) -> Result<Self, Error> {
        fs::create_dir_all(Self::path())?;

        Ok(Self {
            score: Score::default(),
            tests_taken: 0,
        })
    }

    /// Save path for SM
    fn path() -> PathBuf {
        dirs::data_local_dir().unwrap().join("arstyper/scores")
    }

    pub fn save_score(&mut self, score: Score) -> Result<(), Error> {
        self.tests_taken += 1;

        if score.valid() {
            let p = &Self::path().join(format!("{}.toml", score.completed.to_string()));
            let mut f = File::create(&p).or(Err(Error::new(
                ErrorKind::NotFound,
                format!("Unable to create file '{}'", p.display()),
            )))?;
            f.write_all(toml::to_string(&score).unwrap().as_bytes())?;
        }

        self.score = score;
        Ok(())
    }
}
