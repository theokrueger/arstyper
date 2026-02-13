//! Loading and parsing of language files
use rand::prelude::*;
use std::{
    fs::{self, File},
    io::{self, BufRead, Lines},
    iter,
    path::PathBuf,
    process,
};

/// Representation of a language file.
pub struct Lang {
    pub inorder: bool,
    inorder_index: usize,
    pub punctuated: bool,
    pub select_one: bool,
    pub select_all: bool,
    pub words: Vec<String>,
}

impl Lang {
    /// Open a language files by its name, assuming it exists.
    pub fn get_by_name(s: &str) -> Self {
        Self::get_by_path(&Self::path().join(s))
    }

    /// Open a language file by actual path, assuming it exists.
    ///
    /// "Words" are delimited by newlines
    /// Flags:
    /// `inorder` - All words be tested in order. Future tests continue from saved point.
    /// `punctuated` - Test words are already punctuated (used for quotations). Punctuation test setting is ignored.
    /// `select_one` - Only test a single line. Overrides word count setting.
    /// `select_all` - Test with the entire language. Overrides word count setting.
    ///
    /// Which could look like:
    /// ```
    /// flag1
    /// flag2
    /// -----BEGIN WORDLIST-----
    /// word1
    /// word2
    /// ...
    /// ```
    pub fn get_by_path(p: &PathBuf) -> Self {
        let f = File::open(&p).unwrap_or_else(|e| {
            println!(
                "Error reading {}: {e}\nSee available languages with the '--list' flag.",
                p.display()
            );
            process::exit(0b1)
        });

        let buf = io::BufReader::new(f).lines().map_while(Result::ok);
        let mut s = Self {
            inorder: false,
            inorder_index: 0,
            punctuated: false,
            select_one: false,
            select_all: false,
            words: Vec::with_capacity(250),
        };

        // separate lang file by header and word list with a keyword
        // slightly less efficient than splitting a buf but non-issue
        let mut header = true;
        for l in buf {
            if header {
                if l == "-----BEGIN WORDLIST-----" {
                    header = false;
                } else {
                    // flags
                    match l.as_str() {
                        "inorder" => s.inorder = true,
                        "punctuated" => s.punctuated = true,
                        "select_one" => s.select_one = true,
                        "select_all" => s.select_all = true,
                        _ => (),
                    }
                }
            } else {
                s.words.push(l);
            }
        }
        // sanity check
        if s.select_one && s.select_all {
            println!(
                "Error reading {}: Language header has mutually exclusive options `select_one` and `select_all`! Please remove at least one of those options to use this language.",
                p.display()
            );
            process::exit(0b1)
        }

        // unimplemented warn
        {
            // TODO implement these lol
            for (b, s) in vec![
                (s.inorder, "inorder"),
                (s.punctuated, "punctuated"),
                (s.select_one, "select_one"),
                (s.select_all, "select_all"),
            ] {
                if b {
                    println!("The flag `{s}` is not yet implemented! Your language file may not behave as expected.");
                }
            }
        }

        return s;
    }

    /// Return list of all language paths.
    pub fn list() -> Vec<PathBuf> {
        fs::read_dir(Self::path())
            .expect("Unable to read language directory")
            .map(|e| e.unwrap().path())
            .filter(|p| p.is_file())
            .collect::<Vec<PathBuf>>()
    }

    /// Path to language dir.
    fn path() -> PathBuf {
        dirs::data_local_dir().unwrap().join("arstyper")
    }

    /// Get n word iterator of this language for tests
    fn gen_words(&self, n: usize) -> impl Iterator<Item = String> {
        std::iter::from_fn(|| -> Option<String> {
            Some(self.words[rand::random_range(0..self.words.len())].clone())
        })
        .take(n)
    }
}
