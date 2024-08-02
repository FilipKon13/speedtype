use std::{
    cmp::min,
    io::{self, stdout},
    iter,
    time::SystemTime,
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
    ind: usize,
}

impl TextManager {
    fn new(text: Vec<char>) -> Self {
        TextManager { text, ind: 0 }
    }
    fn get_widget<'a>(&self) -> TestLine<'a> {
        TestLine {
            line: Line::raw(self.text.iter().collect::<String>()),
            cursor_ind: self.ind as u16,
        }
    }
    fn handle_key(&mut self, key: KeyCode) {
        if let Some(&c) = self.text.get(self.ind) {
            if key == KeyCode::Char(c) {
                self.ind += 1;
            } else if key == KeyCode::Backspace {
                self.ind = self.ind.saturating_sub(1);
            }
        }
    }
    fn get_correct(&self) -> usize {
        self.ind
    }
}

struct TestLine<'a> {
    line: Line<'a>,
    cursor_ind: u16,
}

impl<'a> TestLine<'a> {
    // use same `area` while rendering
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
            text_manager: TextManager::new(iter::repeat('a').take(50).collect()),
        }
    }
    fn handle_events(&mut self) -> io::Result<bool> {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == event::KeyEventKind::Press {
                    self.text_manager.handle_key(key.code);
                    if key.code == KeyCode::Char('q') {
                        return Ok(true);
                    }
                }
            }
        }
        Ok(false)
    }
    fn milis_elapsed(&self) -> Result<u128, std::time::SystemTimeError> {
        self.start.elapsed().map(|dur| dur.as_millis())
    }
    fn gauge_percent(&self) -> u16 {
        let res = self.milis_elapsed().unwrap_or(0) * 100 / 60000;
        return min(res, 100u128).try_into().unwrap();
    }
    fn wpm(&self) -> u16 {
        if let Ok(milis) = self.milis_elapsed() {
            if milis != 0 {
                return (self.text_manager.get_correct() as u128 * 12000 / milis) as u16;
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
            "0".into(),
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

            frame.render_widget(main_block, frame.size());
            frame.render_widget(gauge, gauge_area);
            frame.render_widget(stat_line, stat_area);
            let (x, y) = text.get_cursor(text_line);
            frame.render_widget(text, text_line);
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
