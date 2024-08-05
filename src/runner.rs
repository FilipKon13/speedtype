use std::{
    cell::Cell,
    cmp::min,
    io::{self, stdout},
    time::{Duration, SystemTime, SystemTimeError},
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
    langs::WordSupplierRandomized,
    layout::{get_testline_width, get_ui_live, get_ui_start},
    text::TextManagerLang,
};

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
    text_manager: TextManagerLang,
}

impl StartedRunner {
    fn new(text_manager: TextManagerLang) -> Self {
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
    fn get_ui(&mut self, width: u16) -> impl FnOnce(&mut Frame) {
        get_ui_live(
            self.time_manager.wpm(self.text_manager.correct() as u16),
            self.text_manager.correct() as u16,
            self.gauge_percent(),
            self.text_manager.get_widget(width),
        )
    }
}

pub enum Runner {
    BeforeStart(TextManagerLang),
    Started(StartedRunner),
    Done(),
}

impl Runner {
    fn get_ui(&mut self, frame_size: Rect) -> Box<dyn FnOnce(&mut Frame)> {
        let width = get_testline_width(frame_size);
        match self {
            Runner::BeforeStart(text_manager) => {
                Box::new(get_ui_start(text_manager.get_widget(width)))
            }
            Runner::Started(runner) => Box::new(runner.get_ui(width)),
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
        Runner::BeforeStart(TextManagerLang::new(
            WordSupplierRandomized::new("english").unwrap(),
        ))
    }
    pub fn run(mut self) -> io::Result<()> {
        enable_raw_mode()?;
        stdout().execute(EnterAlternateScreen)?;
        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        loop {
            terminal.draw(self.get_ui(terminal.size()?))?;
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
