use crate::{
    langs::{WordSupplier, WordSupplierBasic, WordSupplierRandomized},
    layout::TestLine,
};

pub struct TextManager<Ws: WordSupplier> {
    word_supplier: Ws,
    text: Vec<char>,
    word_index: Vec<usize>,
    user_text: Vec<char>,
    correct: usize,
}

impl<Ws: WordSupplier> TextManager<Ws> {
    pub fn new(word_supplier: Ws) -> Self {
        TextManager {
            word_supplier,
            text: vec![],
            word_index: vec![],
            user_text: vec![],
            correct: 0,
        }
    }
    fn get(&mut self, index: usize) -> usize {
        while self.word_index.len() <= index {
            self.word_index.push(self.text.len());
            self.text.extend(self.word_supplier.get_word());
            self.text.push(' ');
        }
        *self.word_index.get(index).unwrap()
    }
    fn get_next_begin(&mut self, mut ind: usize, width: usize) -> usize {
        let end = self.get(ind) + width;
        while self.get(ind + 1) <= end {
            ind += 1;
        }
        ind
    }
    fn widget_data(&mut self, width: usize) -> (&[char], &[char], &[char]) {
        let mut begin = 0usize;
        let mut end = self.get_next_begin(0, width);
        if begin == end {
            return (&[], &[], &[]);
        }
        let ind = self.user_text.len();
        while !(self.get(begin) <= ind && ind < self.get(end)) {
            begin = end;
            end = self.get_next_begin(end, width);
        }
        let next_end = self.get_next_begin(end, width);
        let start = self.get(begin);
        let mid = self.get(end);
        let fin = self.get(next_end);
        (
            &self.text[start..mid],
            &self.text[mid..fin],
            &self.user_text[start..],
        )
    }
    pub fn get_widget<'a>(&mut self, width: usize) -> TestLine<'a> {
        let (line, next_line, user_text) = self.widget_data(width);
        TestLine::new(line, next_line, user_text)
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_widget_correct_width() {
        let mut text_manager = TextManager::new(WordSupplierRandomized::new("english").unwrap());
        for width in 1..1000 {
            let (line, _, _) = text_manager.widget_data(width);
            assert!(line.len() <= width);
        }
    }

    #[test]
    fn short_width() {
        let mut text_manager = TextManager::new(WordSupplierRandomized::new("english").unwrap());
        let (line, _, _) = text_manager.widget_data(1);
        assert!(line.is_empty());
    }

    #[test]
    fn max_width_achieved() {
        let mut text_manager = TextManager::new(WordSupplierRandomized::new("english").unwrap());
        for width in 1..1000 {
            let (line, _, _) = text_manager.widget_data(width);
            if line.len() == width {
                return;
            }
        }
        assert!(false);
    }
}
