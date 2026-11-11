use crate::config::GRAVITY;
use crate::math::Vec2;

const MAX_FALL_SPEED: f32 = 1400.0;
const AIR_DRAG: f32 = 0.15;

#[derive(Debug, Clone)]
pub struct Physics {
    pub velocity: Vec2,
    pub grounded: bool,
}

impl Default for Physics {
    fn default() -> Self {
        Self {
            velocity: Vec2::default(),
            grounded: false,
        }
    }
}

impl Physics {
    pub fn apply_gravity(&mut self, dt: f32) {
        if self.grounded {
            self.velocity.y = self.velocity.y.min(0.0);
            return;
        }
        self.velocity.y += GRAVITY * dt;
        if self.velocity.y > MAX_FALL_SPEED {
            self.velocity.y = MAX_FALL_SPEED;
        }
        self.velocity.x *= (1.0 - AIR_DRAG * dt).max(0.0);
    }

    pub fn integrate(&self, position: Vec2, dt: f32) -> Vec2 {
        Vec2::new(
            position.x + self.velocity.x * dt,
            position.y + self.velocity.y * dt,
        )
    }

    pub fn set_horizontal(&mut self, speed: f32) {
        self.velocity.x = speed;
    }

    pub fn stop_horizontal(&mut self) {
        self.velocity.x = 0.0;
    }

    pub fn throw(&mut self, vx: f32, vy: f32) {
        self.grounded = false;
        self.velocity = Vec2::new(vx, vy);
    }

    pub fn bounce_land(&mut self) {
        self.velocity.y = 0.0;
        self.velocity.x *= 0.35;
        self.grounded = true;
    }

    pub fn leave_ground(&mut self) {
        self.grounded = false;
    }
}
