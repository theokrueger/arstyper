use std::{
    fs::{self, File},
    io::{self, BufRead},
    path::PathBuf,
    process,
};

pub struct Lang {
    pub inorder: bool,
    pub punctuated: bool,
    pub words: Vec<String>,
}

impl Lang {
    pub fn get_by_name(s: &str) -> Self {
        Self::get_by_path(&Self::path().join(s))
    }

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
            punctuated: false,
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
                        _ => (),
                    }
                }
            } else {
                s.words.push(l);
            }
        }

        return s;
    }

    /// Return list of all language paths
    pub fn list() -> Vec<PathBuf> {
        fs::read_dir(Self::path())
            .expect("Unable to read language directory")
            .map(|e| e.unwrap().path())
            .filter(|p| p.is_file())
            .collect::<Vec<PathBuf>>()
    }

    /// Path to language dir
    fn path() -> PathBuf {
        dirs::data_local_dir().unwrap().join("arstyper")
    }
}
