use std::time::{Instant, Duration};

pub struct Clock {
    accumulator: u128,
    delta: u128,
    last_update: Instant,
}

impl Clock {
    pub fn new(delta: Duration) -> Self {
        Self {
            accumulator: 0,
            delta: delta.as_nanos(),
            last_update: Instant::now(),
        }
    }

    pub fn update(&mut self) -> bool {
        let now = Instant::now();

        let accumulated = (now - self.last_update).as_nanos();
        self.accumulator += accumulated;


        self.last_update = now;

        if self.accumulator >= self.delta {
            self.accumulator -= self.delta;
            true
        } else {
            false
        }
    }
}
