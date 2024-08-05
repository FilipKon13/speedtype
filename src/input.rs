use std::time::{Duration, SystemTime};

use ratatui::crossterm::event::{poll, read, Event, KeyCode, KeyEventKind};

pub fn read_key() -> std::io::Result<Option<KeyCode>> {
    let end_time = SystemTime::now()
        .checked_add(Duration::from_millis(100))
        .unwrap();
    loop {
        if poll(
            end_time
                .duration_since(SystemTime::now())
                .unwrap_or_default(),
        )? {
            if let Event::Key(key) = read()? {
                if key.kind == KeyEventKind::Press {
                    return Ok(Some(key.code));
                }
            }
        } else {
            return Ok(None);
        }
    }
}

pub fn read_key_block() -> std::io::Result<KeyCode> {
    loop {
        if let Event::Key(key) = read()? {
            if key.kind == KeyEventKind::Press {
                return Ok(key.code);
            }
        }
    }
}
