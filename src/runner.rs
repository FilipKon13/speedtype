use std::{
    cell::Cell,
    cmp::min,
    io::{self, stdout},
    time::{Duration, SystemTime, SystemTimeError},
    vec,
};

use ratatui::{
    crossterm::{
        event::KeyCode,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    },
    prelude::*,
};

use crate::{
    input::read_key,
    langs,
    layout::{get_ui_live, get_ui_start, TestLine},
};

pub struct TextManager {
    text: Vec<char>,
    user_text: Vec<char>,
    correct: usize,
}

impl TextManager {
    fn _new(text: Vec<char>) -> Self {
        TextManager {
            text,
            user_text: vec![],
            correct: 0,
        }
    }
    fn new_english(max_width: u16) -> Self {
        TextManager {
            text: langs::text_language(max_width, "english").unwrap(),
            user_text: vec![],
            correct: 0,
        }
    }
    fn get_widget<'a>(&self) -> TestLine<'a> {
        TestLine::new(&self.text, &self.user_text)
    }
    fn handle_char(&mut self, u: char) {
        if let Some(&c) = self.text.get(self.user_text.len()) {
            self.user_text.push(u);
            if c == u {
                self.correct += 1;
            }
        }
    }
    fn handle_backspace(&mut self) {
        if let Some(u) = self.user_text.pop() {
            if let Some(&c) = self.text.get(self.user_text.len()) {
                if c == u {
                    self.correct -= 1;
                }
            }
        }
    }
    fn correct(&self) -> usize {
        self.correct
    }
}

struct TimeManager {
    start: SystemTime,
    last_wpm_update: Cell<SystemTime>,
    last_wpm: Cell<u16>,
}

impl TimeManager {
    fn new(time: SystemTime) -> Self {
        TimeManager {
            start: time,
            last_wpm_update: Cell::new(time),
            last_wpm: Cell::new(0),
        }
    }
    fn milis_elapsed(&self) -> Result<u128, SystemTimeError> {
        self.start.elapsed().map(|dur| dur.as_millis())
    }
    fn current_wpm(&self, correct_letters: u16) -> u16 {
        if let Ok(milis) = self.milis_elapsed() {
            if milis != 0 {
                return (correct_letters as u128 * 12000 / milis) as u16;
            }
        }
        0
    }
    fn wpm(&self, correct_letters: u16) -> u16 {
        if let Ok(expired) = self
            .last_wpm_update
            .get()
            .elapsed()
            .map(|t| t >= Duration::from_secs(1))
        {
            if expired {
                self.last_wpm.set(self.current_wpm(correct_letters));
                self.last_wpm_update.set(SystemTime::now());
            }
            return self.last_wpm.get();
        }
        0
    }
}

enum RunnerAction {
    Reset,
    Continue,
    End,
}

pub struct StartedRunner {
    time_manager: TimeManager,
    text_manager: TextManager,
}

impl StartedRunner {
    fn new(text_manager: TextManager) -> Self {
        StartedRunner {
            time_manager: TimeManager::new(SystemTime::now()),
            text_manager,
        }
    }
    fn handle_events(&mut self) -> io::Result<RunnerAction> {
        let next_state = if let Some(key) = read_key()? {
            match key {
                KeyCode::Char(u) => {
                    self.text_manager.handle_char(u);
                    RunnerAction::Continue
                }
                KeyCode::Backspace => {
                    self.text_manager.handle_backspace();
                    RunnerAction::Continue
                }
                KeyCode::Esc => RunnerAction::End,
                KeyCode::Tab => RunnerAction::Reset,
                _ => RunnerAction::Continue,
            }
        } else {
            RunnerAction::Continue
        };
        Ok(next_state)
    }
    fn gauge_percent(&self) -> u16 {
        let res = self.time_manager.milis_elapsed().unwrap_or(0) * 100 / 60000;
        min(res, 100u128).try_into().unwrap()
    }
    fn get_ui(&self) -> impl FnOnce(&mut Frame) {
        get_ui_live(
            self.time_manager.wpm(self.text_manager.correct() as u16),
            self.text_manager.correct as u16,
            self.gauge_percent(),
            self.text_manager.get_widget(),
        )
    }
}

pub enum Runner {
    BeforeStart(TextManager),
    Started(StartedRunner),
    Done(),
}

impl Runner {
    fn get_ui(&self) -> Box<dyn FnOnce(&mut Frame)> {
        match self {
            Runner::BeforeStart(text_manager) => Box::new(get_ui_start(text_manager.get_widget())),
            Runner::Started(runner) => Box::new(runner.get_ui()),
            Runner::Done() => unreachable!(),
        }
    }
    fn handle_events(mut self) -> io::Result<Self> {
        let next_state = match self {
            Runner::BeforeStart(mut text_manager) => {
                if let Ok(Some(key)) = read_key() {
                    match key {
                        KeyCode::Char(c) => {
                            text_manager.handle_char(c);
                            Runner::Started(StartedRunner::new(text_manager))
                        }
                        KeyCode::Esc => Runner::Done(),
                        KeyCode::Tab => Runner::new(),
                        _ => Runner::BeforeStart(text_manager),
                    }
                } else {
                    Runner::BeforeStart(text_manager)
                }
            }
            Runner::Started(ref mut runner) => match runner.handle_events()? {
                RunnerAction::Continue => self,
                RunnerAction::End => Runner::Done(),
                RunnerAction::Reset => Runner::new(),
            },
            Runner::Done() => unreachable!(),
        };
        Ok(next_state)
    }
    pub fn new() -> Self {
        Runner::BeforeStart(TextManager::new_english(50))
    }
    pub fn run(mut self) -> io::Result<()> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        loop {
            terminal.draw(self.get_ui())?;
            self = self.handle_events()?;
            if let Runner::Done() = self {
                break;
            }
        }

        disable_raw_mode()?;
        stdout().execute(LeaveAlternateScreen)?;
        Ok(())
    }
}

impl Default for Runner {
    fn default() -> Self {
        Self::new()
    }
}
