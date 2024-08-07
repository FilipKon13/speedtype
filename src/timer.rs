use std::{
    cell::Cell,
    time::{Duration, SystemTime},
};

pub fn wpm_from_letters(letters: usize, time: Duration) -> f64 {
    letters as f64 * 12000f64 / time.as_millis() as f64
}

pub struct TimeManager {
    start: SystemTime,
    duration: Duration,
    last_wpm_update: Cell<SystemTime>,
    last_wpm: Cell<usize>,
}

impl TimeManager {
    pub fn new(duration: Duration) -> Self {
        let start = SystemTime::now();
        TimeManager {
            start,
            duration,
            last_wpm_update: Cell::new(start),
            last_wpm: Cell::new(0),
        }
    }
    pub fn time_expired(&self) -> bool {
        if let Ok(duration) = self.start.elapsed() {
            return duration > self.duration;
        }
        false
    }
    pub fn percent_elapsed(&self) -> u16 {
        if let Ok(milis) = self.start.elapsed().map(|t| t.as_millis() as f64) {
            let total = self.duration.as_millis() as f64;
            return (milis * 100f64 / total) as u16;
        }
        0
    }
    pub fn duration(&self) -> Duration {
        self.duration
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
