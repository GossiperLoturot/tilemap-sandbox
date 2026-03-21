const TICK_PER_SECS: u64 = 24;

#[derive(Debug, Clone, Default)]
pub struct TimeStorage {
    tick: u64,
    temporary: f32,
}

impl TimeStorage {
    pub fn new() -> Self {
        Default::default()
    }

    #[inline]
    pub fn get_tick_per_secs(&self) -> u64 {
        TICK_PER_SECS
    }

    #[inline]
    pub fn get_tick(&self) -> u64 {
        self.tick
    }

    pub fn process(&mut self, delta_secs: f32) {
        self.temporary += delta_secs;

        let tick = (self.temporary * TICK_PER_SECS as f32) as u32;
        self.tick = self.tick.wrapping_add(tick as u64);

        self.temporary -= tick as f32 / TICK_PER_SECS as f32;
    }
}
