#[derive(Debug, Clone, Default)]
pub struct TickStore {
    tick: u64,
    stack_tick: u64,
    stack_delta_secs: f32,
}

impl TickStore {
    const TICK_PER_SECS: u64 = 24;

    #[inline]
    pub fn per_secs(&self) -> u64 {
        Self::TICK_PER_SECS
    }

    #[inline]
    pub fn get(&self) -> u64 {
        self.tick
    }

    pub fn forward(&mut self, delta_secs: f32) {
        self.stack_delta_secs += delta_secs;

        while self.stack_delta_secs >= 1.0 {
            self.stack_delta_secs -= 1.0;
            self.stack_tick = self.stack_tick.wrapping_add(Self::TICK_PER_SECS);
        }

        let recent_tick = (self.stack_delta_secs * Self::TICK_PER_SECS as f32) as u64;
        self.tick = self.stack_tick.wrapping_add(recent_tick);
    }
}
