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
    input::{read_key, read_key_block},
    langs::WordSupplierRandomized,
    layout::{get_testline_width, get_ui_live, get_ui_start, get_ui_welcome},
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

enum GameAction {
    Reset,
    Continue,
    End,
}

pub struct StartedGame {
    time_manager: TimeManager,
    text_manager: TextManagerLang,
}

impl StartedGame {
    fn new(text_manager: TextManagerLang) -> Self {
        StartedGame {
            time_manager: TimeManager::new(SystemTime::now()),
            text_manager,
        }
    }
    fn handle_events(&mut self) -> io::Result<GameAction> {
        let next_state = if let Some(key) = read_key()? {
            match key {
                KeyCode::Char(u) => {
                    self.text_manager.handle_char(u);
                    GameAction::Continue
                }
                KeyCode::Backspace => {
                    self.text_manager.handle_backspace();
                    GameAction::Continue
                }
                KeyCode::Esc => GameAction::End,
                KeyCode::Tab => GameAction::Reset,
                _ => GameAction::Continue,
            }
        } else {
            GameAction::Continue
        };
        Ok(next_state)
    }
    fn gauge_percent(&self) -> u16 {
        let res = self.time_manager.milis_elapsed().unwrap_or(0) * 100 / 60000;
        min(res, 100u128).try_into().unwrap()
    }
    fn get_ui(&mut self, width: usize) -> impl FnOnce(&mut Frame) {
        get_ui_live(
            self.time_manager.wpm(self.text_manager.correct() as u16),
            self.text_manager.correct() as u16,
            self.gauge_percent(),
            self.text_manager.get_widget(width),
        )
    }
}

enum GameState {
    BeforeStart(TextManagerLang),
    Started(StartedGame),
    Done(),
}

impl GameState {
    fn new() -> Self {
        GameState::BeforeStart(TextManagerLang::new(
            WordSupplierRandomized::new("english").unwrap(),
        ))
    }
    fn get_ui(&mut self, frame_size: Rect) -> Box<dyn FnOnce(&mut Frame)> {
        let width = get_testline_width(frame_size) as usize;
        match self {
            GameState::BeforeStart(text_manager) => {
                Box::new(get_ui_start(text_manager.get_widget(width)))
            }
            GameState::Started(runner) => Box::new(runner.get_ui(width)),
            GameState::Done() => unreachable!(),
        }
    }
    fn handle_events(mut self) -> io::Result<Self> {
        let next_state = match self {
            GameState::BeforeStart(mut text_manager) => {
                if let Ok(Some(key)) = read_key() {
                    match key {
                        KeyCode::Char(c) => {
                            text_manager.handle_char(c);
                            GameState::Started(StartedGame::new(text_manager))
                        }
                        KeyCode::Esc => GameState::Done(),
                        KeyCode::Tab => GameState::new(),
                        _ => GameState::BeforeStart(text_manager),
                    }
                } else {
                    GameState::BeforeStart(text_manager)
                }
            }
            GameState::Started(ref mut runner) => match runner.handle_events()? {
                GameAction::Continue => self,
                GameAction::End => GameState::Done(),
                GameAction::Reset => GameState::new(),
            },
            GameState::Done() => unreachable!(),
        };
        Ok(next_state)
    }
}

enum RunnerState {
    StartScreen,
    LiveGame(GameState),
    EndGameScreen,
    Done,
}

pub struct Runner<'a, B: Backend> {
    terminal: &'a mut Terminal<B>,
    state: RunnerState,
}

impl<'a, B: Backend> Runner<'a, B> {
    fn get_ui(&mut self, frame_size: Rect) -> Box<dyn FnOnce(&mut Frame)> {
        match self.state {
            RunnerState::StartScreen | RunnerState::EndGameScreen => Box::new(get_ui_welcome()),
            RunnerState::LiveGame(ref mut game_state) => game_state.get_ui(frame_size),
            RunnerState::Done => unreachable!(),
        }
    }
    fn handle_events(mut self) -> io::Result<Self> {
        match self.state {
            RunnerState::StartScreen => loop {
                let key = read_key_block()?;
                if key == KeyCode::Tab {
                    self.state = RunnerState::LiveGame(GameState::new());
                    break;
                }
            },
            RunnerState::LiveGame(game_state) => {
                let new_game_state = game_state.handle_events()?;
                if let GameState::Done() = new_game_state {
                    self.state = RunnerState::EndGameScreen;
                } else {
                    self.state = RunnerState::LiveGame(new_game_state);
                }
            }
            RunnerState::EndGameScreen => loop {
                let key = read_key_block()?;
                if key == KeyCode::Tab {
                    self.state = RunnerState::StartScreen;
                    break;
                }
                if key == KeyCode::Esc {
                    self.state = RunnerState::Done;
                    break;
                }
            },
            RunnerState::Done => unreachable!(),
        }
        Ok(self)
    }
    fn is_done(&self) -> bool {
        if let RunnerState::Done = self.state {
            true
        } else {
            false
        }
    }
    pub fn new(terminal: &'a mut Terminal<B>) -> Self {
        Runner {
            terminal,
            state: RunnerState::StartScreen,
        }
    }
    pub fn run(mut self) -> io::Result<()> {
        loop {
            let frame = self.get_ui(self.terminal.size()?);
            self.terminal.draw(frame)?;
            self = self.handle_events()?;
            if self.is_done() {
                return Ok(());
            }
        }
    }
}

pub fn start_game() -> io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let runner = Runner::new(&mut terminal);
    runner.run()?;

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
