pub const DAY_LENGTH_SECONDS: f32 = 1_200.0;

#[derive(Clone, Copy, Debug, Default)]
pub struct WorldClock {
    elapsed_seconds: f32,
}

impl WorldClock {
    pub fn tick(&mut self, dt: f32) {
        self.elapsed_seconds = (self.elapsed_seconds + dt.max(0.0)) % DAY_LENGTH_SECONDS;
    }

    /// Fraction of a full day, from 0.0 at noon to 0.5 at midnight.
    pub fn day_phase(&self) -> f32 {
        self.elapsed_seconds / DAY_LENGTH_SECONDS
    }

    /// Smooth daylight intensity between night and full daylight.
    pub fn daylight(&self) -> f32 {
        (0.5 + 0.5 * (self.day_phase() * std::f32::consts::TAU).cos()).clamp(0.0, 1.0)
    }

    pub fn sky_rgb(&self) -> [f32; 3] {
        let daylight = self.daylight();
        let night = [0.012, 0.018, 0.055];
        let day = [0.48, 0.70, 0.91];
        [
            night[0] + (day[0] - night[0]) * daylight,
            night[1] + (day[1] - night[1]) * daylight,
            night[2] + (day[2] - night[2]) * daylight,
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_starts_at_daylight() {
        let clock = WorldClock::default();
        assert!((clock.daylight() - 1.0).abs() < 0.0001);
    }

    #[test]
    fn half_day_reaches_night() {
        let mut clock = WorldClock::default();
        clock.tick(DAY_LENGTH_SECONDS * 0.5);
        assert!(clock.daylight() < 0.001);
    }

    #[test]
    fn clock_wraps_after_full_day() {
        let mut clock = WorldClock::default();
        clock.tick(DAY_LENGTH_SECONDS + 10.0);
        assert!((clock.day_phase() - 10.0 / DAY_LENGTH_SECONDS).abs() < 0.0001);
    }
}
