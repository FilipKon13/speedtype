use crate::{
    langs::{WordSupplier, WordSupplierBasic, WordSupplierRandomized},
    layout::TestLine,
};

pub struct TextManager<Ws: WordSupplier> {
    word_supplier: Ws,
    text: Vec<char>,
    user_text: Vec<char>,
    correct: usize,
    begin_index: usize,
}

impl<Ws: WordSupplier> TextManager<Ws> {
    pub fn new(word_supplier: Ws) -> Self {
        TextManager {
            word_supplier,
            text: vec![],
            user_text: vec![],
            correct: 0,
            begin_index: 0,
        }
    }
    fn get_line_index(&mut self, begin: usize, width: usize) -> usize {
        while begin + width > self.text.len() {
            self.text.extend(self.word_supplier.get_word());
            self.text.push(' ');
        }
        let mut res = begin + width;
        while res > begin {
            if *self.text.get(res - 1).unwrap() == ' ' {
                break;
            }
            res -= 1;
        }
        res
    }
    // TODO: fix character removal
    pub fn get_widget<'a>(&mut self, width: u16) -> TestLine<'a> {
        let mut begin_next = self.begin_index;
        while begin_next <= self.user_text.len() {
            self.begin_index = begin_next;
            begin_next = self.get_line_index(self.begin_index, width as usize);
        }
        let end_next = self.get_line_index(begin_next, width as usize);
        TestLine::new(
            &self.text[self.begin_index..begin_next],
            &self.text[begin_next..end_next],
            &self.user_text[self.begin_index..],
        )
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

pub type TextManagerBasic = TextManager<WordSupplierBasic>;
pub type TextManagerLang = TextManager<WordSupplierRandomized>;
