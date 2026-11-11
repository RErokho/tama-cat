#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

impl Vec2 {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn hypot(self) -> f32 {
        self.x.hypot(self.y)
    }
}

pub fn clamp(v: f32, min: f32, max: f32) -> f32 {
    v.max(min).min(max)
}
