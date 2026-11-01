/// Survival health and food state. Values use the familiar 0..=20 scale.
#[derive(Clone, Copy, Debug)]
pub struct Vitals {
    pub health: f32,
    pub food: f32,
    exhaustion: f32,
    starvation_timer: f32,
    regeneration_timer: f32,
}

impl Default for Vitals {
    fn default() -> Self {
        Self {
            health: 20.0,
            food: 20.0,
            exhaustion: 0.0,
            starvation_timer: 0.0,
            regeneration_timer: 0.0,
        }
    }
}

impl Vitals {
    pub fn damage(&mut self, amount: f32) {
        self.health = (self.health - amount.max(0.0)).clamp(0.0, 20.0);
    }

    pub fn heal(&mut self, amount: f32) {
        self.health = (self.health + amount.max(0.0)).clamp(0.0, 20.0);
    }

    pub fn add_food(&mut self, amount: f32) {
        self.food = (self.food + amount.max(0.0)).clamp(0.0, 20.0);
    }

    pub fn is_alive(&self) -> bool {
        self.health > 0.0
    }

    pub fn tick(&mut self, dt: f32, sprinting: bool) {
        let dt = dt.clamp(0.0, 0.1);
        if sprinting {
            self.exhaustion += dt * 0.08;
        }

        if self.exhaustion >= 4.0 {
            self.exhaustion -= 4.0;
            self.food = (self.food - 1.0).max(0.0);
        }

        if self.food >= 18.0 && self.health < 20.0 {
            self.regeneration_timer += dt;
            if self.regeneration_timer >= 4.0 {
                self.regeneration_timer = 0.0;
                self.heal(1.0);
                self.exhaustion += 6.0;
            }
        } else {
            self.regeneration_timer = 0.0;
        }

        if self.food <= 0.0 {
            self.starvation_timer += dt;
            if self.starvation_timer >= 4.0 {
                self.starvation_timer = 0.0;
                self.damage(1.0);
            }
        } else {
            self.starvation_timer = 0.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_and_food_are_clamped() {
        let mut vitals = Vitals::default();
        vitals.damage(50.0);
        vitals.add_food(50.0);
        assert_eq!(vitals.health, 0.0);
        assert_eq!(vitals.food, 20.0);
        assert!(!vitals.is_alive());
    }

    #[test]
    fn health_regenerates_when_food_is_high() {
        let mut vitals = Vitals::default();
        vitals.damage(2.0);
        for _ in 0..41 { vitals.tick(0.1, false); }
        assert!(vitals.health > 18.0);
    }

    #[test]
    fn starvation_causes_damage_over_time() {
        let mut vitals = Vitals::default();
        vitals.food = 0.0;
        for _ in 0..41 { vitals.tick(0.1, false); }
        assert!(vitals.health < 20.0);
    }
}
