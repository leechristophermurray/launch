use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use crate::domain::ports::ITimeService;

#[derive(Clone, Copy, Debug, PartialEq)]
enum TimeMode {
    Timer { start: Instant, duration: Duration, original_duration: Duration },
    Pomodoro { start: Instant, duration: Duration },
    Stopwatch { start: Instant },
    PausedTimer { remaining: Duration, original_duration: Duration },
    PausedPomodoro { remaining: Duration },
    PausedStopwatch { elapsed: Duration },
    Idle,
}

pub struct TimeAdapter {
    state: Arc<Mutex<TimeMode>>,
}

impl TimeAdapter {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(TimeMode::Idle)),
        }
    }
}

impl ITimeService for TimeAdapter {
    fn start_timer(&self, duration_secs: u64) {
        let mut state = self.state.lock().unwrap();
        let duration = Duration::from_secs(duration_secs);
        *state = TimeMode::Timer { 
            start: Instant::now(), 
            duration, 
            original_duration: duration 
        };
    }

    fn start_pomodoro(&self) {
        let mut state = self.state.lock().unwrap();
        // Standard 25 minutes
        *state = TimeMode::Pomodoro { 
            start: Instant::now(), 
            duration: Duration::from_secs(25 * 60) 
        };
    }

    fn start_stopwatch(&self) {
        let mut state = self.state.lock().unwrap();
        *state = TimeMode::Stopwatch { start: Instant::now() };
    }

    fn get_status(&self) -> (String, bool) {
        let mut state = self.state.lock().unwrap();
        match *state {
            TimeMode::Timer { start, duration, .. } => {
                let elapsed = start.elapsed();
                if elapsed >= duration {
                    *state = TimeMode::Idle;
                    ("Done".to_string(), false)
                } else {
                    let remaining = duration - elapsed;
                    let secs = remaining.as_secs();
                    (format!("{:02}:{:02}", secs / 60, secs % 60), true)
                }
            },
            TimeMode::Pomodoro { start, duration } => {
                let elapsed = start.elapsed();
                if elapsed >= duration {
                    *state = TimeMode::Idle;
                    ("Focus Done".to_string(), false)
                } else {
                    let remaining = duration - elapsed;
                    let secs = remaining.as_secs();
                    (format!("🍅 {:02}:{:02}", secs / 60, secs % 60), true)
                }
            },
            TimeMode::Stopwatch { start } => {
                let elapsed = start.elapsed();
                let secs = elapsed.as_secs();
                (format!("{:02}:{:02}", secs / 60, secs % 60), true)
            },
            TimeMode::PausedTimer { remaining, .. } => {
                let secs = remaining.as_secs();
                (format!("⏸ {:02}:{:02}", secs / 60, secs % 60), true)
            },
            TimeMode::PausedPomodoro { remaining } => {
                let secs = remaining.as_secs();
                (format!("⏸ 🍅 {:02}:{:02}", secs / 60, secs % 60), true)
            },
            TimeMode::PausedStopwatch { elapsed } => {
                 let secs = elapsed.as_secs();
                (format!("⏸ {:02}:{:02}", secs / 60, secs % 60), true)
            },
            TimeMode::Idle => (String::new(), false),
        }
    }

    fn stop(&self) {
        let mut state = self.state.lock().unwrap();
        *state = TimeMode::Idle;
    }

    fn toggle_pause(&self) {
        let mut state = self.state.lock().unwrap();
        *state = match *state {
            TimeMode::Timer { start, duration, original_duration } => {
                let elapsed = start.elapsed();
                let remaining = if elapsed < duration { duration - elapsed } else { Duration::ZERO };
                TimeMode::PausedTimer { remaining, original_duration }
            },
            TimeMode::Pomodoro { start, duration } => {
                let elapsed = start.elapsed();
                let remaining = if elapsed < duration { duration - elapsed } else { Duration::ZERO };
                TimeMode::PausedPomodoro { remaining }
            },
            TimeMode::Stopwatch { start } => {
                TimeMode::PausedStopwatch { elapsed: start.elapsed() }
            },
            TimeMode::PausedTimer { remaining, original_duration } => {
                TimeMode::Timer { 
                    start: Instant::now(), 
                    duration: remaining, 
                    original_duration 
                }
            },
            TimeMode::PausedPomodoro { remaining } => {
                TimeMode::Pomodoro { 
                    start: Instant::now(), 
                    duration: remaining 
                }
            },
            TimeMode::PausedStopwatch { elapsed } => {
                // To restore stopwatch, we need start time to appear as 'elapsed' ago
                TimeMode::Stopwatch { 
                    start: Instant::now() - elapsed 
                }
            },
            _ => *state,
        };
    }

    fn restart(&self) {
        let mut state = self.state.lock().unwrap();
        *state = match *state {
            TimeMode::Timer { original_duration, .. } | TimeMode::PausedTimer { original_duration, .. } => {
                TimeMode::Timer { 
                    start: Instant::now(), 
                    duration: original_duration, 
                    original_duration 
                }
            },
            TimeMode::Pomodoro { .. } | TimeMode::PausedPomodoro { .. } => {
                 TimeMode::Pomodoro { 
                    start: Instant::now(), 
                    duration: Duration::from_secs(25 * 60) 
                }
            },
            TimeMode::Stopwatch { .. } | TimeMode::PausedStopwatch { .. } => {
                TimeMode::Stopwatch { start: Instant::now() }
            },
            _ => *state,
        };
    }
}
