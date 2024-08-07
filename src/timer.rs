use std::{
    cell::Cell,
    time::{Duration, SystemTime, SystemTimeError},
};

pub fn wpm_from_letters(letters: usize, time: Duration) -> f64 {
    letters as f64 * 12000f64 / time.as_millis() as f64
}

pub struct TimeManager {
    start: SystemTime,
    last_wpm_update: Cell<SystemTime>,
    last_wpm: Cell<usize>,
}

impl TimeManager {
    pub fn new() -> Self {
        let time = SystemTime::now();
        TimeManager {
            start: time,
            last_wpm_update: Cell::new(time),
            last_wpm: Cell::new(0),
        }
    }
    pub fn time_expired(&self) -> bool {
        if let Ok(duration) = self.start.elapsed() {
            return duration > Duration::from_secs(60);
        }
        false
    }
    pub fn milis_elapsed(&self) -> Result<u128, SystemTimeError> {
        Ok(self.start.elapsed()?.as_millis())
    }
    fn real_wpm(&self, correct_letters: usize) -> usize {
        if let Ok(duration) = self.start.elapsed() {
            return wpm_from_letters(correct_letters, duration) as usize;
        }
        0
    }
    pub fn wpm(&self, correct_letters: usize) -> usize {
        if let Ok(expired) = self
            .last_wpm_update
            .get()
            .elapsed()
            .map(|t| t >= Duration::from_secs(1))
        {
            if expired {
                self.last_wpm.set(self.real_wpm(correct_letters));
                self.last_wpm_update.set(SystemTime::now());
            }
            return self.last_wpm.get();
        }
        0
    }
}
