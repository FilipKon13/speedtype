use std::io::{self, stdout};

use ratatui::{
    crossterm::event::KeyCode,
    prelude::{Backend, CrosstermBackend},
    Frame, Terminal,
};

use crate::{
    game::{GameStats, LiveGame, NextState},
    input::read_key_block,
    layout::GameStatsScreen,
    welcome::{StartScreen, StartScreenAction},
};

pub fn start_game() -> io::Result<()> {
    use ratatui::crossterm::{
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
        ExecutableCommand,
    };
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    App::new().run(&mut terminal)?;

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

#[derive(Clone)]
pub struct GameOptions {
    pub time: u32,
}

enum AppState {
    StartScreen(StartScreen),
    LiveGame(LiveGame),
    EndGameScreen(GameStatsScreen),
}

pub struct App {
    options: GameOptions,
    state: AppState,
}

impl App {
    fn handle_events(mut self) -> io::Result<Option<Self>> {
        let state = match self.state {
            AppState::StartScreen(ref mut start_screen) => match start_screen.handle_events() {
                StartScreenAction::Continue => {
                    self.options = start_screen.get_options();
                    self.state
                }
                StartScreenAction::Quit => return Ok(None),
                StartScreenAction::StartGame => {
                    self.options = start_screen.get_options();
                    AppState::LiveGame(LiveGame::new(self.options.clone()))
                }
            },
            AppState::LiveGame(live_game) => match live_game.handle_events()? {
                NextState::LiveGame(live_game) => AppState::LiveGame(live_game),
                NextState::Exit => return Ok(Some(App::new())),
                NextState::GameEnded(GameStats { wpm, acc }) => {
                    AppState::EndGameScreen(GameStatsScreen::new(wpm, acc))
                }
                NextState::Restart => AppState::LiveGame(LiveGame::new(self.options.clone())),
            },
            AppState::EndGameScreen(_) => loop {
                let key = read_key_block()?;
                if key == KeyCode::Tab {
                    break AppState::LiveGame(LiveGame::new(self.options.clone()));
                }
                if key == KeyCode::Esc {
                    return Ok(None);
                }
            },
        };
        Ok(Some(App { state, ..self }))
    }
    pub fn new() -> Self {
        let options = GameOptions { time: 60 };
        App {
            options: options.clone(),
            state: AppState::StartScreen(StartScreen::new(options)),
        }
    }
    pub fn run<B: Backend>(mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        loop {
            let frame = |frame: &mut Frame| {
                let mut cursor = None;
                frame.render_stateful_widget(&mut self, frame.size(), &mut cursor);
                if let Some((x, y)) = cursor {
                    frame.set_cursor(x, y);
                }
            };
            terminal.draw(frame)?;
            match self.handle_events()? {
                Some(s) => self = s,
                None => return Ok(()),
            }
        }
    }
}

mod widget {
    use ratatui::{
        buffer::Buffer,
        layout::{Alignment, Rect},
        widgets::StatefulWidget,
    };

    use super::{App, AppState};

    impl StatefulWidget for &mut App {
        type State = Option<(u16, u16)>;
        fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
            use ratatui::widgets::*;

            let main_block = Block::new()
                .borders(Borders::TOP)
                .title(block::Title::from("SpeedType").alignment(Alignment::Center));

            let inner_area = main_block.inner(area);

            main_block.render(area, buf);

            match &mut self.state {
                AppState::StartScreen(start_screen) => start_screen.render(inner_area, buf),
                AppState::EndGameScreen(game_stats) => game_stats.render(inner_area, buf),
                AppState::LiveGame(live_game) => live_game.render(inner_area, buf, state),
            }
        }
    }
}
