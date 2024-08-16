use crate::input::read_key_block;
use tui_menu::{MenuEvent, MenuItem, MenuState};

pub enum StartScreenAction {
    ChangeTime(u32),
    Continue,
    Quit,
    StartGame,
}

pub struct StartScreen {
    menu: MenuState<u32>,
}

impl StartScreen {
    pub fn new() -> Self {
        StartScreen {
            menu: MenuState::<u32>::new(vec![MenuItem::group(
                "Time",
                vec![
                    MenuItem::item("10 s", 10),
                    MenuItem::item("30 s", 30),
                    MenuItem::item("60 s", 60),
                ],
            )]),
        }
    }
    pub fn handle_events(&mut self) -> StartScreenAction {
        use ratatui::crossterm::event::KeyCode::*;
        use StartScreenAction::*;
        if let Ok(key) = read_key_block() {
            match key {
                Enter => self.menu.select(),
                Down => self.menu.down(),
                Up => self.menu.up(),
                Left => self.menu.left(),
                Right => self.menu.right(),
                Esc => {
                    if self.menu.highlight().is_none()
                        || self.menu.highlight().unwrap().data.is_none()
                    {
                        return Quit;
                    }
                    self.menu.reset();
                }
                Tab => return StartGame,
                _ => {}
            }
        }

        let mut action = Continue;

        for e in self.menu.drain_events() {
            match e {
                MenuEvent::Selected(time) => {
                    self.menu.reset();
                    action = ChangeTime(time);
                }
            }
        }
        action
    }
}

impl Default for StartScreen {
    fn default() -> Self {
        Self::new()
    }
}

mod widget {
    use ratatui::prelude::*;
    use tui_menu::Menu;

    use crate::app::GameOptions;

    use super::StartScreen;

    impl StatefulWidget for &mut StartScreen {
        type State = GameOptions;
        fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
            use Constraint::*;
            let [top, _, bot] = Layout::vertical([Length(1), Percentage(20), Fill(1)]).areas(area);
            let option_area =
                Layout::horizontal([Fill(1), Percentage(50), Fill(1)]).areas::<3>(bot)[1];
            let [left, right] =
                Layout::horizontal([Percentage(50), Percentage(50)]).areas(option_area);
            Line::raw("Press Tab to start")
                .bold()
                .centered()
                .render(top, buf);
            Line::raw(format!("Time: {} s", state.time))
                .bold()
                .left_aligned()
                .render(left, buf);
            Menu::new().render(right, buf, &mut self.menu);
        }
    }
}
