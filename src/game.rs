use ratatui::crossterm::event::KeyCode;
use std::cmp::min;
use std::time::Duration;

use crate::{
    input::read_key,
    langs::WordSupplierRandomized,
    text::TextManagerLang,
    timer::{wpm_from_letters, TimeManager},
};

enum GameAction {
    Reset,
    Continue,
    Quit,
    End(GameStats),
}

struct StartedGame {
    time_manager: TimeManager,
    text_manager: TextManagerLang,
}

impl StartedGame {
    fn new(text_manager: TextManagerLang) -> Self {
        StartedGame {
            time_manager: TimeManager::new(),
            text_manager,
        }
    }
    fn handle_events(&mut self) -> std::io::Result<GameAction> {
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
}

enum GameState {
    BeforeStart(TextManagerLang),
    Started(StartedGame),
}

impl GameState {
    fn new() -> Self {
        GameState::BeforeStart(TextManagerLang::new(
            WordSupplierRandomized::new("english").unwrap(),
        ))
    }
}

pub struct LiveGame {
    state: GameState,
}

pub struct GameStats {
    pub wpm: f64,
    pub acc: f64,
}

pub enum NextState {
    LiveGame(LiveGame),
    GameEnded(GameStats),
    Exit,
}

impl LiveGame {
    pub fn new() -> Self {
        LiveGame {
            state: GameState::new(),
        }
    }
    pub fn handle_events(mut self) -> std::io::Result<NextState> {
        let state = match self.state {
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
                GameAction::Continue => self.state,
                GameAction::Quit => return Ok(NextState::Exit),
                GameAction::Reset => GameState::new(),
                GameAction::End(game_stats) => return Ok(NextState::GameEnded(game_stats)),
            },
        };
        Ok(NextState::LiveGame(LiveGame { state }))
    }
}

mod widget {
    use ratatui::{
        buffer::Buffer,
        layout::Rect,
        widgets::{StatefulWidget, Widget},
    };

    use crate::layout::{get_ui_live_widgets, AppLayout};

    use super::{GameState, LiveGame};

    impl StatefulWidget for &mut LiveGame {
        type State = Option<(u16, u16)>;
        fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
            let AppLayout {
                gauge_area,
                stat_area,
                text_area,
            } = AppLayout::new(area);

            let text_manager = match &mut self.state {
                GameState::BeforeStart(text_manager) => text_manager,
                GameState::Started(started_game) => &mut started_game.text_manager,
            };
            text_manager.render(text_area, buf, state);

            if let GameState::Started(started_game) = &mut self.state {
                let correct = started_game.text_manager.correct();
                let wpm = started_game.time_manager.wpm(correct);
                let gauge_percent = started_game.gauge_percent();
                let (gauge, stat_line) = get_ui_live_widgets(wpm, correct, gauge_percent);
                gauge.render(gauge_area, buf);
                stat_line.render(stat_area, buf);
            }
        }
    }
}
