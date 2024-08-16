use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

use rand::{rngs::ThreadRng, thread_rng, Rng};

pub trait WordSupplier {
    fn get_word(&mut self) -> Vec<char>;
}

pub struct WordSupplierRandomized {
    words: Vec<Vec<char>>,
    rng: ThreadRng,
}

impl WordSupplierRandomized {
    pub fn new(lang: &str) -> io::Result<Self> {
        let mut file = {
            let mut path = ["languages", lang].into_iter().collect::<PathBuf>();
            path.set_extension("txt");
            File::open(path)?
        };
        let mut buf = String::new();
        file.read_to_string(&mut buf)?;
        let words: Vec<Vec<char>> = buf
            .split_ascii_whitespace()
            .filter(|s| s.len() > 1)
            .map(|s| s.to_lowercase().chars().collect::<Vec<_>>())
            .collect();
        Ok(WordSupplierRandomized {
            words,
            rng: thread_rng(),
        })
    }
}

impl WordSupplier for WordSupplierRandomized {
    fn get_word(&mut self) -> Vec<char> {
        let index = self.rng.gen::<usize>() % self.words.len();
        self.words.get(index).unwrap().clone()
    }
}

pub struct WordSupplierBasic {
    text: Vec<char>,
}

impl WordSupplier for WordSupplierBasic {
    fn get_word(&mut self) -> Vec<char> {
        self.text.clone()
    }
}

impl WordSupplierBasic {
    pub fn new(text: Vec<char>) -> Self {
        WordSupplierBasic { text }
    }
}
