use crate::{
    langs::{WordSupplier, WordSupplierBasic, WordSupplierRandomized},
    layout::{TestLine, TextWidgetGenerator},
};

pub struct TextManager<Ws: WordSupplier> {
    word_supplier: Ws,
    text: Vec<char>,
    user_text: Vec<char>,
    correct: usize,
}

// TODO: should be removed later to fix hardcoded 50
fn generate_text<Ws: WordSupplier>(ws: &mut Ws) -> Vec<char> {
    let mut text = vec![];
    loop {
        let word = ws.get_word();
        if text.len() + word.len() < 50 {
            if !text.is_empty() {
                text.push(' ')
            }
            text.extend(word);
        } else {
            return text;
        }
    }
}

impl TextManager<WordSupplierBasic> {
    pub fn new(text: Vec<char>) -> Self {
        TextManager::from_supplier(WordSupplierBasic::new(text))
    }
}

impl TextManager<WordSupplierRandomized> {
    pub fn new() -> Self {
        TextManager::from_supplier(WordSupplierRandomized::new("english").unwrap())
    }
}

impl<Ws: WordSupplier> TextWidgetGenerator for TextManager<Ws> {
    fn get_widget<'a>(&self, _width: u16) -> TestLine<'a> {
        TestLine::new(&self.text, &self.user_text)
    }
}

impl<Ws: WordSupplier> TextManager<Ws> {
    fn from_supplier(mut word_supplier: Ws) -> Self {
        let text = generate_text(&mut word_supplier);
        TextManager {
            word_supplier,
            text,
            user_text: vec![],
            correct: 0,
        }
    }
    pub fn handle_char(&mut self, u: char) {
        if let Some(&c) = self.text.get(self.user_text.len()) {
            self.user_text.push(u);
            if c == u {
                self.correct += 1;
            }
        }
    }
    pub fn handle_backspace(&mut self) {
        if let Some(u) = self.user_text.pop() {
            if let Some(&c) = self.text.get(self.user_text.len()) {
                if c == u {
                    self.correct -= 1;
                }
            }
        }
    }
    pub fn correct(&self) -> usize {
        self.correct
    }
}

impl Default for TextManager<WordSupplierRandomized> {
    fn default() -> Self {
        Self::new()
    }
}

pub type TextManagerBasic = TextManager<WordSupplierBasic>;
pub type TextManagerLang = TextManager<WordSupplierRandomized>;
