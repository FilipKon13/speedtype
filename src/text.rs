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
    fn begin_of_word(&mut self, index: usize) -> usize {
        while self.word_index.len() <= index {
            self.word_index.push(self.text.len());
            self.text.extend(self.word_supplier.get_word());
            self.text.push(' ');
        }
        *self.word_index.get(index).unwrap()
    }
    fn next_line_begin(&mut self, mut ind: usize, width: usize) -> usize {
        let end = self.begin_of_word(ind) + width;
        while self.begin_of_word(ind + 1) <= end {
            ind += 1;
        }
        ind
    }
    fn widget_data(&mut self, width: usize) -> WidgetData {
        let mut begin = 0usize;
        let mut end = self.next_line_begin(0, width);
        if begin == end {
            return WidgetData {
                line: &[],
                next_line: &[],
                user_text: &[],
            };
        }
        let ind = self.user_text.len();
        while !(self.begin_of_word(begin) <= ind && ind < self.begin_of_word(end)) {
            begin = end;
            end = self.next_line_begin(end, width);
        }
        let next_end = self.next_line_begin(end, width);
        let start = self.begin_of_word(begin);
        let mid = self.begin_of_word(end);
        let finish = self.begin_of_word(next_end);
        WidgetData {
            line: &self.text[start..mid],
            next_line: &self.text[mid..finish],
            user_text: &self.user_text[start..],
        }
    }
    pub fn get_widget<'a>(&mut self, width: usize) -> TestLine<'a> {
        let WidgetData {
            line,
            next_line,
            user_text,
        } = self.widget_data(width);
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

struct WidgetData<'a> {
    line: &'a [char],
    next_line: &'a [char],
    user_text: &'a [char],
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_widget_correct_width() {
        let mut text_manager = TextManager::new(WordSupplierRandomized::new("english").unwrap());
        for width in 1..1000 {
            let line = text_manager.widget_data(width).line;
            assert!(line.len() <= width);
        }
    }

    #[test]
    fn short_width() {
        let mut text_manager = TextManager::new(WordSupplierRandomized::new("english").unwrap());
        let line = text_manager.widget_data(1).line;
        assert!(line.is_empty());
    }

    #[test]
    fn max_width_achieved() {
        let mut text_manager = TextManager::new(WordSupplierRandomized::new("english").unwrap());
        for width in 1..1000 {
            let line = text_manager.widget_data(width).line;
            if line.len() == width {
                return;
            }
        }
        assert!(false);
    }
}
