use runner::Runner;

pub mod input;
pub mod langs;
pub mod layout;
pub mod runner;
pub mod text;

fn initialize_panic_handler() {
    std::panic::set_hook(Box::new(|panic_info| {
        ratatui::crossterm::execute!(
            std::io::stderr(),
            ratatui::crossterm::terminal::LeaveAlternateScreen
        )
        .unwrap();
        ratatui::crossterm::terminal::disable_raw_mode().unwrap();
        better_panic::Settings::auto()
            .most_recent_first(false)
            .lineno_suffix(true)
            .create_panic_handler()(panic_info);
    }));
}

fn main() -> std::io::Result<()> {
    initialize_panic_handler();
    Runner::new().run()
}
