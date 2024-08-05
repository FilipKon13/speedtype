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
    layout::{get_testline_width, get_ui_game_end, get_ui_live, get_ui_start, get_ui_welcome},
    text::TextManagerLang,
    util::wpm_from_letters,
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
    fn time_expired(&self) -> bool {
        if let Ok(duration) = self.start.elapsed() {
            return duration > Duration::from_secs(60);
        }
        false
    }
    fn milis_elapsed(&self) -> Result<u128, SystemTimeError> {
        Ok(self.start.elapsed()?.as_millis())
    }
    fn current_wpm(&self, correct_letters: u16) -> u16 {
        if let Ok(duration) = self.start.elapsed() {
            return wpm_from_letters(correct_letters as usize, duration) as u16;
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
    Quit,
    End(GameStats),
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
        if self.time_manager.time_expired() {
            return Ok(GameAction::End(GameStats {
                wpm: wpm_from_letters(self.text_manager.correct(), Duration::from_secs(60)),
                acc: self.text_manager.correct() as f64,
            }));
        }
        let action = if let Some(key) = read_key()? {
            match key {
                KeyCode::Char(u) => {
                    self.text_manager.handle_char(u);
                    GameAction::Continue
                }
                KeyCode::Backspace => {
                    self.text_manager.handle_backspace();
                    GameAction::Continue
                }
                KeyCode::Esc => GameAction::Quit,
                KeyCode::Tab => GameAction::Reset,
                _ => GameAction::Continue,
            }
        } else {
            GameAction::Continue
        };
        Ok(action)
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

struct GameStats {
    wpm: f64,
    acc: f64,
}

enum GameState {
    BeforeStart(TextManagerLang),
    Started(StartedGame),
}

enum NextState {
    GameState(GameState),
    GameEnded(GameStats),
    Exit,
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
        }
    }
    fn handle_events(mut self) -> io::Result<NextState> {
        let game_state = match self {
            GameState::BeforeStart(mut text_manager) => {
                if let Some(key) = read_key()? {
                    match key {
                        KeyCode::Char(c) => {
                            text_manager.handle_char(c);
                            GameState::Started(StartedGame::new(text_manager))
                        }
                        KeyCode::Esc => return Ok(NextState::Exit),
                        KeyCode::Tab => GameState::new(),
                        _ => GameState::BeforeStart(text_manager),
                    }
                } else {
                    GameState::BeforeStart(text_manager)
                }
            }
            GameState::Started(ref mut runner) => match runner.handle_events()? {
                GameAction::Continue => self,
                GameAction::Quit => return Ok(NextState::Exit),
                GameAction::Reset => GameState::new(),
                GameAction::End(game_stats) => return Ok(NextState::GameEnded(game_stats)),
            },
        };
        Ok(NextState::GameState(game_state))
    }
}

enum RunnerState {
    StartScreen,
    LiveGame(GameState),
    EndGameScreen(GameStats),
}

pub struct Runner<'a, B: Backend> {
    terminal: &'a mut Terminal<B>,
    state: RunnerState,
}

impl<'a, B: Backend> Runner<'a, B> {
    fn get_ui(&mut self, frame_size: Rect) -> Box<dyn FnOnce(&mut Frame)> {
        match self.state {
            RunnerState::StartScreen => Box::new(get_ui_welcome()),
            RunnerState::LiveGame(ref mut game_state) => game_state.get_ui(frame_size),
            RunnerState::EndGameScreen(GameStats { wpm, acc }) => {
                Box::new(get_ui_game_end(wpm, acc))
            }
        }
    }
    fn handle_events(mut self) -> io::Result<Option<Self>> {
        match self.state {
            RunnerState::StartScreen | RunnerState::EndGameScreen(_) => loop {
                let key = read_key_block()?;
                if key == KeyCode::Tab {
                    self.state = RunnerState::LiveGame(GameState::new());
                    break;
                }
                if key == KeyCode::Esc {
                    return Ok(None);
                }
            },
            RunnerState::LiveGame(game_state) => match game_state.handle_events()? {
                NextState::GameState(game_state) => {
                    self.state = RunnerState::LiveGame(game_state);
                }
                NextState::Exit => self.state = RunnerState::StartScreen,
                NextState::GameEnded(game_stats) => {
                    self.state = RunnerState::EndGameScreen(game_stats)
                }
            },
        }
        Ok(Some(self))
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
            match self.handle_events()? {
                Some(s) => self = s,
                None => return Ok(()),
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
