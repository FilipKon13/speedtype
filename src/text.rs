use crate::langs::{WordSupplier, WordSupplierBasic, WordSupplierRandomized};

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
        let mut begin = [0usize; 4];
        begin[1] = self.next_line_begin(begin[0], width);
        if begin[0] == begin[1] {
            return WidgetData::empty();
        }
        begin[2] = self.next_line_begin(begin[1], width);
        begin[3] = self.next_line_begin(begin[2], width);
        let ind = self.user_text.len();
        if self.begin_of_word(begin[0]) <= ind && ind < self.begin_of_word(begin[1]) {
            let inds = begin.map(|i| self.begin_of_word(i));
            return WidgetData {
                prev_line: &self.text[inds[0]..inds[1]],
                line: &self.text[inds[1]..inds[2]],
                next_line: &self.text[inds[2]..inds[3]],
                prev_user_text: &self.user_text[inds[0]..],
                user_text: &[],
            };
        }
        while !(self.begin_of_word(begin[1]) <= ind && ind < self.begin_of_word(begin[2])) {
            begin[0] = begin[1];
            begin[1] = begin[2];
            begin[2] = begin[3];
            begin[3] = self.next_line_begin(begin[3], width);
        }
        let inds = begin.map(|i| self.begin_of_word(i));
        WidgetData {
            prev_line: &self.text[inds[0]..inds[1]],
            line: &self.text[inds[1]..inds[2]],
            next_line: &self.text[inds[2]..inds[3]],
            prev_user_text: &self.user_text[inds[0]..inds[1]],
            user_text: &self.user_text[inds[1]..],
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

pub type TextManagerBasic = TextManager<WordSupplierBasic>;
pub type TextManagerLang = TextManager<WordSupplierRandomized>;

struct WidgetData<'a> {
    prev_line: &'a [char],
    line: &'a [char],
    next_line: &'a [char],
    prev_user_text: &'a [char],
    user_text: &'a [char],
}

impl<'a> WidgetData<'a> {
    fn empty() -> Self {
        WidgetData {
            prev_line: &[],
            line: &[],
            next_line: &[],
            prev_user_text: &[],
            user_text: &[],
        }
    }
}

mod widget {
    use ratatui::{
        buffer::Buffer,
        layout::Rect,
        widgets::{StatefulWidget, Widget},
    };

    use crate::{langs::WordSupplier, layout::TestLines};

    use super::{TextManager, WidgetData};

    impl<Ws: WordSupplier> StatefulWidget for &mut TextManager<Ws> {
        type State = Option<(u16, u16)>;
        fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
            let WidgetData {
                prev_line,
                line,
                next_line,
                prev_user_text,
                user_text,
            } = self.widget_data(area.width as usize);
            let text = TestLines::new(prev_line, line, next_line, prev_user_text, user_text);
            text.render(area, buf);
            let cursor = if user_text.is_empty() && prev_user_text.len() < prev_line.len() {
                (area.left() + prev_user_text.len() as u16, area.top())
            } else {
                (area.left() + user_text.len() as u16, area.top() + 1)
            };
            *state = Some(cursor);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn get_widget_correct_width() {
        let mut text_manager = TextManager::new(WordSupplierRandomized::new("english").unwrap());
        for width in 1..1000 {
            let line = text_manager.widget_data(width).prev_line;
            assert!(line.len() <= width);
        }
    }

    #[test]
    fn too_short_width() {
        let mut text_manager = TextManager::new(WordSupplierRandomized::new("english").unwrap());
        let line = text_manager.widget_data(1).prev_line;
        assert!(line.is_empty());
    }

    #[test]
    fn max_width_achieved() {
        let mut text_manager = TextManager::new(WordSupplierRandomized::new("english").unwrap());
        for width in 1..1000 {
            let line = text_manager.widget_data(width).prev_line;
            if line.len() == width {
                return;
            }
        }
        panic!();
    }
}
