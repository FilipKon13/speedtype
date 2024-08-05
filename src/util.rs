use std::time::Duration;

pub fn wpm_from_letters(letters: usize, time: Duration) -> f64 {
    letters as f64 * 12000f64 / time.as_millis() as f64
}
