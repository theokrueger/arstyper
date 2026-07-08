//! Loading and parsing of language files
use std::{
    cmp::min,
    collections::VecDeque,
    fs::{self, File},
    io::{self, BufRead, Error, ErrorKind},
    path::{Path, PathBuf},
};

/// Representation of a language file.
pub struct Lang {
    pub name: String,
    pub words: Vec<String>,
}

impl Lang {
    /// Open a language files by its name, assuming it exists.
    pub fn get_by_name(s: &str) -> Result<Self, Error> {
        Self::get_by_path(&Self::path().join(s), s)
    }

    /// Open a language file by actual path, assuming it exists.
    ///
    /// "Words" are delimited by newlines
    /// Which could look like:
    /// ```
    /// word1
    /// word2
    /// ...
    /// wordN
    /// ```
    pub fn get_by_path(p: &Path, name: &str) -> Result<Self, Error> {
        let f = File::open(&p).or(Err(Error::new(
            ErrorKind::NotFound,
            format!("No such language '{name}'"),
        )))?;

        let buf = io::BufReader::new(f).lines().map_while(Result::ok);
        Ok(Self {
            name: name.to_string(),
            words: buf.collect(),
        })
    }

    /// Return list of all language paths.
    pub fn list() -> Result<Vec<PathBuf>, Error> {
        Ok(fs::read_dir(Self::path())?
            .map(|e| e.unwrap().path())
            .filter(|p| p.is_file())
            .collect::<Vec<PathBuf>>())
    }

    /// Path to language dir.
    fn path() -> PathBuf {
        dirs::data_local_dir()
            .expect("No data directory on system")
            .join("arstyper/tests")
    }

    /// Get n word iterator of this language for tests
    /// Has a "repeat window" of 4 such that no word shows up more than once every 5 words
    pub fn gen_words(&self, n: usize) -> impl Iterator<Item = String> + '_ {
        let n_win = min(4, self.words.len().saturating_sub(1));
        let mut window: VecDeque<String> = VecDeque::with_capacity(n_win);
        std::iter::from_fn(move || -> Option<String> {
            loop {
                let s = self.words[rand::random_range(0..self.words.len())].clone();
                if !window.contains(&s) {
                    window.push_back(s.clone());
                    if window.len() > n_win {
                        window.pop_front();
                    }
                    return Some(s);
                }
            }
        })
        .take(n)
    }
}
