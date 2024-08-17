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
    let mut game_options = GameOptions { time: 60 };
    App::new(&mut game_options).run(&mut terminal)?;

    // TODO: save options

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}

pub struct GameOptions {
    pub time: u32,
}

enum AppState {
    StartScreen(StartScreen),
    LiveGame(LiveGame),
    EndGameScreen(GameStatsScreen),
}

pub struct App<'a> {
    options: &'a mut GameOptions,
    state: AppState,
}

impl<'a> App<'a> {
    fn handle_events(mut self) -> io::Result<Option<Self>> {
        let state = match self.state {
            AppState::StartScreen(ref mut start_screen) => match start_screen.handle_events() {
                StartScreenAction::Continue => self.state,
                StartScreenAction::Quit => return Ok(None),
                StartScreenAction::StartGame => {
                    AppState::LiveGame(LiveGame::new(self.options.time))
                }
                StartScreenAction::ChangeTime(time) => {
                    self.options.time = time;
                    self.state
                }
            },
            AppState::LiveGame(live_game) => match live_game.handle_events()? {
                NextState::LiveGame(live_game) => AppState::LiveGame(live_game),
                NextState::Exit => AppState::StartScreen(StartScreen::new()),
                NextState::GameEnded(GameStats { wpm, acc }) => {
                    AppState::EndGameScreen(GameStatsScreen::new(wpm, acc))
                }
                NextState::Restart => AppState::LiveGame(LiveGame::new(self.options.time)),
            },
            AppState::EndGameScreen(_) => loop {
                let key = read_key_block()?;
                if key == KeyCode::Tab {
                    break AppState::LiveGame(LiveGame::new(self.options.time));
                }
                if key == KeyCode::Esc {
                    break AppState::StartScreen(StartScreen::new());
                }
            },
        };
        Ok(Some(App { state, ..self }))
    }
    pub fn new(options: &'a mut GameOptions) -> Self {
        App {
            options,
            state: AppState::StartScreen(StartScreen::new()),
        }
    }
    pub fn run<B: Backend>(mut self, terminal: &mut Terminal<B>) -> io::Result<()> {
        let mut cursor = None;
        loop {
            let frame = |frame: &mut Frame| {
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

    impl<'a> StatefulWidget for &mut App<'a> {
        type State = Option<(u16, u16)>;
        fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
            use ratatui::widgets::*;
            *state = None;

            let main_block = Block::new()
                .borders(Borders::TOP)
                .title(block::Title::from("SpeedType").alignment(Alignment::Center));

            let inner_area = main_block.inner(area);

            main_block.render(area, buf);

            match &mut self.state {
                AppState::StartScreen(start_screen) => {
                    start_screen.render(inner_area, buf, self.options)
                }
                AppState::EndGameScreen(game_stats) => game_stats.render(inner_area, buf),
                AppState::LiveGame(live_game) => live_game.render(inner_area, buf, state),
            }
        }
    }
}
