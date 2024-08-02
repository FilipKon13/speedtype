use std::{
    cmp::min,
    fs::File,
    io::{self, stdout, Read},
    iter,
    path::PathBuf,
    time::{Duration, SystemTime, SystemTimeError},
    vec,
};

use ratatui::{
    crossterm::{
        event::{self, Event, KeyCode},
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    prelude::*,
    widgets::*,
};

struct TextManager {
    text: Vec<char>,
    user_text: Vec<char>,
    correct: usize,
}

impl TextManager {
    fn new(text: Vec<char>) -> Self {
        TextManager {
            text,
            user_text: vec![],
            correct: 0,
        }
    }
    fn new_english(max_width: u16) -> Self {
        let mut file =
            File::open(["languages", "english.txt"].iter().collect::<PathBuf>()).unwrap();
        let mut buf = String::new();
        file.read_to_string(&mut buf).unwrap();
        let words = buf
            .split_ascii_whitespace()
            .map(|s| s.to_lowercase().chars().collect::<Vec<_>>())
            .collect::<Vec<_>>();
        let mut text = vec![];
        for word in words.iter() {
            if text.len() + word.len() < max_width as usize {
                if !text.is_empty() {
                    text.push(' ');
                }
                text.extend(word);
            } else {
                break;
            }
        }
        TextManager {
            text,
            user_text: vec![],
            correct: 0,
        }
    }
    fn get_widget<'a>(&self) -> TestLine<'a> {
        TestLine {
            line: Line::from(
                self.text
                    .iter()
                    .zip(
                        self.user_text
                            .clone()
                            .into_iter()
                            .map(Some)
                            .chain(iter::repeat(None)),
                    )
                    .map(|(&c, u)| {
                        Span::raw(c.to_string()).style(match u {
                            Some(u) => {
                                if c == u {
                                    Color::Green
                                } else {
                                    Color::Red
                                }
                            }
                            None => Color::Blue,
                        })
                    })
                    .collect::<Vec<_>>(),
            ),
            cursor_ind: self.user_text.len() as u16,
        }
    }
    fn handle_key(&mut self, key: KeyCode) {
        match key {
            KeyCode::Char(u) => {
                if let Some(&c) = self.text.get(self.user_text.len()) {
                    self.user_text.push(u);
                    if c == u {
                        self.correct += 1;
                    }
                }
            }
            KeyCode::Backspace => {
                if let Some(u) = self.user_text.pop() {
                    if let Some(&c) = self.text.get(self.user_text.len()) {
                        if c == u {
                            self.correct -= 1;
                        }
                    }
                }
            }
            _ => {}
        }
    }
    fn correct(&self) -> usize {
        self.correct
    }
}

struct TestLine<'a> {
    line: Line<'a>,
    cursor_ind: u16,
}

impl<'a> TestLine<'a> {
    /// use same `area` as in `Frame::render()`
    fn get_cursor(&self, area: Rect) -> (u16, u16) {
        (area.left() + self.cursor_ind, area.top())
    }
}

impl<'a> Widget for TestLine<'a> {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.line.render(area, buf)
    }
}

pub struct Runner {
    start: SystemTime,
    text_manager: TextManager,
}

impl Runner {
    pub fn new() -> Self {
        Runner {
            start: SystemTime::now(),
            text_manager: TextManager::new_english(50),
        }
    }
    fn handle_events(&mut self) -> io::Result<bool> {
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    self.text_manager.handle_key(key.code);
                    if key.code == KeyCode::Esc {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }
    fn milis_elapsed(&self) -> Result<u128, SystemTimeError> {
        self.start.elapsed().map(|dur| dur.as_millis())
    }
    fn gauge_percent(&self) -> u16 {
        let res = self.milis_elapsed().unwrap_or(0) * 100 / 60000;
        return min(res, 100u128).try_into().unwrap();
    }
    fn wpm(&self) -> u16 {
        if let Ok(milis) = self.milis_elapsed() {
            if milis != 0 {
                return (self.text_manager.correct() as u128 * 12000 / milis) as u16;
            }
        }
        0
    }
    fn get_ui(&self) -> impl FnOnce(&mut Frame) {
        let gauge = Gauge::default()
            .gauge_style(Style::default().fg(Color::Blue).bg(Color::Red))
            .percent(self.gauge_percent())
            .label(Span::default())
            .use_unicode(true);
        let stat_line = Line::from(vec![
            "WPM: ".bold(),
            self.wpm().to_string().into(),
            " Acc: ".bold(),
            self.text_manager.correct().to_string().into(),
        ])
        .left_aligned();
        let text = self.text_manager.get_widget();
        move |frame| {
            let main_block = Block::new()
                .borders(Borders::TOP)
                .title(block::Title::from("SpeedType").alignment(Alignment::Center));
            let [gauge_area, _, stat_area, _, text_line, _] = Layout::vertical([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Percentage(20),
                Constraint::Length(1),
                Constraint::Min(0),
            ])
            .areas(main_block.inner(frame.size()));
            let [_, text_area, _] = Layout::horizontal([
                Constraint::Min(0),
                Constraint::Length(text.line.width() as u16),
                Constraint::Min(0),
            ])
            .areas(text_line);

            frame.render_widget(main_block, frame.size());
            frame.render_widget(gauge, gauge_area);
            frame.render_widget(stat_line, stat_area);
            let (x, y) = text.get_cursor(text_area);
            frame.render_widget(text, text_area);
            frame.set_cursor(x, y);
        }
    }
    pub fn run(mut self) -> io::Result<()> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        loop {
            terminal.draw(self.get_ui())?;
            if self.handle_events()? {
                break;
            }
        }

        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }
}
