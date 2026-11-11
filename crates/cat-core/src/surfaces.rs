use crate::config::{MAX_JUMP_DOWN_PX, MAX_JUMP_UP_PX};
use crate::math::Vec2;
use crate::types::{Surface, SurfaceKind};

pub fn find_support_under(point: Vec2, surfaces: &[Surface], max_drop: f32) -> Option<&Surface> {
    let mut best: Option<&Surface> = None;
    let mut best_dy = f32::INFINITY;
    for surface in surfaces {
        if surface.kind == SurfaceKind::WindowSide {
            continue;
        }
        if point.x < surface.x - 4.0 || point.x > surface.x + surface.width + 4.0 {
            continue;
        }
        let dy = surface.y - point.y;
        if dy < -2.0 || dy > max_drop {
            continue;
        }
        if dy < best_dy {
            best_dy = dy;
            best = Some(surface);
        }
    }
    best
}

pub fn is_on_window_top(support: Option<&Surface>) -> bool {
    matches!(support.map(|s| s.kind), Some(SurfaceKind::WindowTop))
}

pub fn find_climbable_side_near<'a>(
    center: Vec2,
    surfaces: &'a [Surface],
    reach_x: f32,
) -> Option<&'a Surface> {
    let mut best: Option<&Surface> = None;
    let mut best_dist = reach_x + 1.0;
    for s in surfaces {
        if s.kind != SurfaceKind::WindowSide {
            continue;
        }
        if s.height < 80.0 {
            continue;
        }
        let sx = s.x + s.width / 2.0;
        let dx = (center.x - sx).abs();
        if dx > reach_x {
            continue;
        }
        if center.y < s.y - 10.0 || center.y > s.y + s.height + 20.0 {
            continue;
        }
        if dx < best_dist {
            best_dist = dx;
            best = Some(s);
        }
    }
    best
}

pub fn find_window_hop_target<'a>(
    foot_x: f32,
    current_y: f32,
    surfaces: &'a [Surface],
    prefer_up: Option<bool>,
) -> Option<&'a Surface> {
    let mut best: Option<&Surface> = None;
    let mut best_score = f32::INFINITY;
    for t in surfaces.iter().filter(|s| s.kind == SurfaceKind::WindowTop) {
        if foot_x < t.x - 8.0 || foot_x > t.x + t.width + 8.0 {
            continue;
        }
        let dy = t.y - current_y;
        if dy.abs() < 50.0 {
            continue;
        }
        if dy < 0.0 && -dy > MAX_JUMP_UP_PX {
            continue;
        }
        if dy > 0.0 && dy > MAX_JUMP_DOWN_PX {
            continue;
        }
        if prefer_up == Some(true) && dy >= 0.0 {
            continue;
        }
        if prefer_up == Some(false) && dy <= 0.0 {
            continue;
        }
        let score = dy.abs() + (foot_x - (t.x + t.width / 2.0)).abs() * 0.05;
        if score < best_score {
            best_score = score;
            best = Some(t);
        }
    }
    best
}
