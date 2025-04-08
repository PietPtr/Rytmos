#[derive(Debug, Clone, Copy)]
pub struct Debouncer {
    goal_stable_time: u32,
    current_stable_time: u32,
    last_state: bool,
    current_stable_state: bool,
    last_stable_state: bool,
}

impl Debouncer {
    pub fn new(stable_time: u32) -> Self {
        Self {
            goal_stable_time: stable_time,
            current_stable_time: 0,
            last_state: false,
            current_stable_state: false,
            last_stable_state: false,
        }
    }

    pub fn update(&mut self, state: bool) {
        if state != self.last_state {
            self.current_stable_time = 0;
        } else {
            self.current_stable_time += 1;
        }

        if self.current_stable_time > self.goal_stable_time {
            self.last_stable_state = self.current_stable_state; // put the last stable state in memory
            self.current_stable_state = state; // this state is now stable
        }

        self.last_state = state
    }

    pub fn is_high(&self) -> Result<bool, DebounceError> {
        if self.current_stable_time > self.goal_stable_time {
            Ok(self.last_state)
        } else {
            Err(DebounceError::NotStable(
                self.goal_stable_time - self.current_stable_time,
            ))
        }
    }

    pub fn is_low(&self) -> Result<bool, DebounceError> {
        if self.current_stable_time > self.goal_stable_time {
            Ok(!self.last_state)
        } else {
            Err(DebounceError::NotStable(
                self.goal_stable_time - self.current_stable_time,
            ))
        }
    }

    pub fn stable_rising_edge(&self) -> bool {
        self.current_stable_state && !self.last_stable_state
    }
}

#[derive(Debug, defmt::Format)]
pub enum DebounceError {
    NotStable(u32),
}
