//! Loading and parsing of language files
use std::{
    fs::{self, File},
    io::{self, BufRead, Error, ErrorKind},
    path::PathBuf,
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
    pub fn get_by_path(p: &PathBuf, name: &str) -> Result<Self, Error> {
        let f = File::open(&p).or(Err(Error::new(
            ErrorKind::NotFound,
            format!("No such language '{name}'"),
        )))?;

        let buf = io::BufReader::new(f).lines().map_while(Result::ok);
        let mut s = Self {
            name: name.to_string(),
            words: Vec::with_capacity(250),
        };

        for l in buf {
            s.words.push(l);
        }

        return Ok(s);
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
        dirs::data_local_dir().unwrap().join("arstyper/tests")
    }

    /// Get n word iterator of this language for tests
    pub fn gen_words(&self, n: usize) -> impl Iterator<Item = String> {
        std::iter::from_fn(|| -> Option<String> {
            Some(self.words[rand::random_range(0..self.words.len())].clone())
        })
        .take(n)
    }
}
