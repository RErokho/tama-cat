use crate::config::{
    EDGE_INSET, MAX_CLIMB_UP_PX, MAX_JUMP_DOWN_PX, MAX_JUMP_UP_PX, MAX_THROW_UP_SPEED,
    PET_FRAME_HEIGHT, PET_FRAME_WIDTH, RUN_SPEED, SIDE_PAD, WALK_SPEED,
};
use crate::math::{clamp, Vec2};
use crate::physics::Physics;
use crate::surfaces::{
    find_climbable_side_near, find_support_under, find_window_hop_target, is_on_window_top,
};
use crate::types::{AnimClip, DisplayInfo, Surface};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocomotionIntent {
    Idle,
    Walk,
    Run,
    Drag,
    Fall,
    Land,
    ClingTop,
    WallLean,
    JumpUp,
    JumpDown,
    LookDown,
    HangDrop,
    ClimbWindow,
}

/// Mid-air pose while `LocomotionIntent::Fall`. Intent stays Fall; only the clip changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallStyle {
    Spread,
    Tumble,
    Curl,
    Flail,
}

impl FallStyle {
    fn clip(self) -> AnimClip {
        // Sheet variants land later; keep classic fall until fallTumble/curl/flail frames exist.
        match self {
            Self::Spread => AnimClip::Fall,
            Self::Tumble => AnimClip::FallTumble,
            Self::Curl => AnimClip::FallCurl,
            Self::Flail => AnimClip::FallFlail,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct DragSample {
    t_ms: u64,
    x: f32,
    y: f32,
}

pub struct MovementEngine {
    pub physics: Physics,
    pub position: Vec2,
    pub facing: i8,
    pub intent: LocomotionIntent,
    foot_offset: f32,
    dragging: bool,
    drag_samples: Vec<DragSample>,
    land_timer: f32,
    fall_style: FallStyle,
    fall_style_timer: f32,
    special_timer: f32,
    wall_lean_cooldown: f32,
    transfer_target: Option<Vec2>,
    climb_target_y: f32,
    last_support: Option<Surface>,
    surfaces: Vec<Surface>,
    displays: Vec<DisplayInfo>,
    pub just_landed: bool,
    now_ms: u64,
}

impl Default for MovementEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl MovementEngine {
    pub fn new() -> Self {
        Self {
            physics: Physics::default(),
            position: Vec2::new(200.0, 200.0),
            facing: 1,
            intent: LocomotionIntent::Idle,
            foot_offset: PET_FRAME_HEIGHT - 4.0,
            dragging: false,
            drag_samples: Vec::new(),
            land_timer: 0.0,
            fall_style: FallStyle::Spread,
            fall_style_timer: 0.0,
            special_timer: 0.0,
            wall_lean_cooldown: 0.0,
            transfer_target: None,
            climb_target_y: 0.0,
            last_support: None,
            surfaces: Vec::new(),
            displays: Vec::new(),
            just_landed: false,
            now_ms: 0,
        }
    }

    pub fn set_clock_ms(&mut self, now_ms: u64) {
        self.now_ms = now_ms;
    }

    pub fn set_displays(&mut self, displays: Vec<DisplayInfo>) {
        self.displays = displays;
    }

    pub fn is_dragging(&self) -> bool {
        self.dragging
    }

    pub fn airborne(&self) -> bool {
        matches!(
            self.intent,
            LocomotionIntent::Fall
                | LocomotionIntent::Drag
                | LocomotionIntent::JumpUp
                | LocomotionIntent::JumpDown
                | LocomotionIntent::HangDrop
        ) || self.dragging
    }

    pub fn special_pose(&self) -> bool {
        matches!(
            self.intent,
            LocomotionIntent::ClingTop
                | LocomotionIntent::WallLean
                | LocomotionIntent::JumpUp
                | LocomotionIntent::JumpDown
                | LocomotionIntent::LookDown
                | LocomotionIntent::HangDrop
                | LocomotionIntent::ClimbWindow
        )
    }

    pub fn begin_drag(&mut self) {
        self.dragging = true;
        self.intent = LocomotionIntent::Drag;
        self.land_timer = 0.0;
        self.special_timer = 0.0;
        self.reset_fall_style();
        self.transfer_target = None;
        self.just_landed = false;
        self.drag_samples.clear();
        self.physics.leave_ground();
        self.physics.velocity = Vec2::default();
        self.push_drag_sample(
            self.position.x + PET_FRAME_WIDTH / 2.0,
            self.position.y + PET_FRAME_HEIGHT / 2.0,
        );
    }

    pub fn drag_to(&mut self, screen: Vec2) {
        self.position = Vec2::new(
            screen.x - PET_FRAME_WIDTH / 2.0,
            screen.y - PET_FRAME_HEIGHT / 2.0,
        );
        self.push_drag_sample(screen.x, screen.y);
    }

    pub fn end_drag(&mut self) {
        self.dragging = false;
        let (vx, vy) = self.estimate_throw_velocity();
        self.drag_samples.clear();
        self.physics.throw(vx, vy);
        self.enter_fall();
        if vx.abs() > 40.0 {
            self.facing = if vx >= 0.0 { 1 } else { -1 };
        }
    }

    pub fn start_walk(&mut self, direction: i8) {
        if self.dragging || self.airborne() || self.special_pose() || self.intent == LocomotionIntent::Land
        {
            return;
        }
        self.facing = direction;
        self.intent = LocomotionIntent::Walk;
        self.physics.set_horizontal(direction as f32 * WALK_SPEED);
    }

    pub fn start_run(&mut self, direction: i8) {
        if self.dragging || self.airborne() || self.special_pose() || self.intent == LocomotionIntent::Land
        {
            return;
        }
        self.facing = direction;
        self.intent = LocomotionIntent::Run;
        self.physics.set_horizontal(direction as f32 * RUN_SPEED);
    }

    pub fn stop(&mut self) {
        if self.dragging || self.intent == LocomotionIntent::Fall || self.special_pose() {
            return;
        }
        self.physics.stop_horizontal();
        if self.physics.grounded && self.intent != LocomotionIntent::Land {
            self.intent = LocomotionIntent::Idle;
        }
    }

    pub fn snap_to_primary(&mut self, d: &DisplayInfo) {
        self.position = Vec2::new(d.left + 200.0, d.work_top + 200.0);
        self.physics.bounce_land();
        self.physics.stop_horizontal();
        self.intent = LocomotionIntent::Idle;
        self.land_timer = 0.0;
        self.special_timer = 0.0;
        self.reset_fall_style();
        self.transfer_target = None;
    }

    pub fn try_wall_lean(&mut self) -> bool {
        if self.dragging
            || self.airborne()
            || self.intent == LocomotionIntent::Land
            || self.wall_lean_cooldown > 0.0
        {
            return false;
        }
        let Some(d) = self.current_display() else {
            return false;
        };
        let cx = self.position.x + PET_FRAME_WIDTH / 2.0;
        let near_left = cx <= d.left + SIDE_PAD + 24.0;
        let near_right = cx >= d.right - SIDE_PAD - 24.0;
        if !near_left && !near_right {
            return false;
        }
        self.facing = if near_right { 1 } else { -1 };
        self.physics.stop_horizontal();
        self.physics.bounce_land();
        self.intent = LocomotionIntent::WallLean;
        self.special_timer = 2.5 + fastrand(0.0, 2.0);
        self.snap_wall_lean_x(&d);
        true
    }

    pub fn try_peek_drop_from_window(&mut self) -> bool {
        if !self.physics.grounded || !is_on_window_top(self.last_support.as_ref()) {
            return false;
        }
        if self.dragging || self.special_pose() {
            return false;
        }
        self.physics.stop_horizontal();
        self.intent = LocomotionIntent::LookDown;
        // Long enough for lookDown-0..2 gradual head-down cycle.
        self.special_timer = 1.85 + fastrand(0.0, 0.9);
        true
    }

    pub fn try_climb_window_side(&mut self) -> bool {
        if self.dragging || self.special_pose() {
            return false;
        }
        let center = Vec2::new(
            self.position.x + PET_FRAME_WIDTH / 2.0,
            self.position.y + PET_FRAME_HEIGHT / 2.0,
        );
        let Some(side) = find_climbable_side_near(center, &self.surfaces, 42.0) else {
            return false;
        };
        let top_y = side.y - self.foot_offset + 4.0;
        if self.position.y <= top_y + 8.0 {
            return false;
        }
        let rise = self.position.y - top_y;
        if rise > MAX_CLIMB_UP_PX {
            return false;
        }
        self.facing = if side.id.contains("-R-") { 1 } else { -1 };
        self.physics.stop_horizontal();
        self.physics.leave_ground();
        self.climb_target_y = top_y;
        self.intent = LocomotionIntent::ClimbWindow;
        self.special_timer = 3.5;
        true
    }

    pub fn try_hop_window(&mut self, prefer_up: Option<bool>) -> bool {
        if self.dragging || self.special_pose() {
            return false;
        }
        let foot_x = self.position.x + PET_FRAME_WIDTH / 2.0;
        let support_y = self
            .last_support
            .as_ref()
            .map(|s| s.y)
            .unwrap_or(self.position.y + self.foot_offset);
        let Some(target) = find_window_hop_target(foot_x, support_y, &self.surfaces, prefer_up)
        else {
            return false;
        };
        let target_pos = Vec2::new(
            clamp(
                target.x + target.width * 0.3 + fastrand(0.0, target.width * 0.4)
                    - PET_FRAME_WIDTH / 2.0,
                target.x - SIDE_PAD,
                target.x + target.width - PET_FRAME_WIDTH + SIDE_PAD,
            ),
            target.y - self.foot_offset,
        );
        self.facing = if target_pos.x >= self.position.x { 1 } else { -1 };
        self.physics.stop_horizontal();
        self.physics.leave_ground();
        self.transfer_target = Some(target_pos);
        self.intent = if target.y < support_y {
            LocomotionIntent::JumpUp
        } else {
            LocomotionIntent::JumpDown
        };
        self.special_timer = 2.5;
        true
    }

    pub fn resolve_anim(&self) -> AnimClip {
        match self.intent {
            LocomotionIntent::Walk => AnimClip::Walk,
            LocomotionIntent::Run => AnimClip::Run,
            LocomotionIntent::Drag => AnimClip::Drag,
            LocomotionIntent::Fall => self.fall_style.clip(),
            LocomotionIntent::Land => AnimClip::Land,
            LocomotionIntent::ClingTop => AnimClip::ClingTop,
            LocomotionIntent::WallLean => AnimClip::WallLean,
            LocomotionIntent::JumpUp => AnimClip::JumpUp,
            LocomotionIntent::JumpDown => AnimClip::JumpDown,
            LocomotionIntent::LookDown => AnimClip::LookDown,
            LocomotionIntent::HangDrop => AnimClip::HangDrop,
            LocomotionIntent::ClimbWindow => AnimClip::ClimbWindow,
            _ => AnimClip::Idle,
        }
    }

    pub fn update(&mut self, dt: f32, surfaces: &[Surface]) {
        self.surfaces = surfaces.to_vec();
        self.just_landed = false;
        if self.wall_lean_cooldown > 0.0 {
            self.wall_lean_cooldown = (self.wall_lean_cooldown - dt).max(0.0);
        }
        if self.dragging {
            return;
        }
        if self.intent == LocomotionIntent::Fall {
            self.tick_fall_style(dt);
        }

        match self.intent {
            LocomotionIntent::LookDown => {
                self.special_timer -= dt;
                if self.special_timer <= 0.0 {
                    self.intent = LocomotionIntent::HangDrop;
                    // Hold inverted hind-hang so it reads as graceful.
                    self.special_timer = 1.15 + fastrand(0.0, 0.55);
                    // hangDrop cells are top-aligned (hind paws at cell top).
                    // Snap so the grip sits on the shelf — not the foot line.
                    let shelf_y = self
                        .last_support
                        .as_ref()
                        .map(|s| s.y)
                        .unwrap_or(self.position.y + self.foot_offset);
                    self.position.y = shelf_y - 4.0;
                    self.physics.leave_ground();
                }
                return;
            }
            LocomotionIntent::HangDrop => {
                self.special_timer -= dt;
                // Soft settle: body slides further below the shelf while gripping.
                self.position.y += 18.0 * dt;
                if self.special_timer <= 0.0 {
                    self.physics.throw(
                        self.facing as f32 * (20.0 + fastrand(0.0, 40.0)),
                        160.0 + fastrand(0.0, 80.0),
                    );
                    // Peek-drop should open as a calm fall, not a tumble spin.
                    self.enter_fall_with(FallStyle::Spread);
                }
                return;
            }
            LocomotionIntent::ClimbWindow => {
                self.update_climb_window(dt);
                return;
            }
            LocomotionIntent::ClingTop => {
                self.update_cling_top(dt);
                return;
            }
            LocomotionIntent::WallLean => {
                if let Some(d) = self.current_display() {
                    self.snap_wall_lean_x(&d);
                }
                self.special_timer -= dt;
                if self.special_timer <= 0.0 {
                    self.finish_wall_lean();
                }
                return;
            }
            LocomotionIntent::JumpUp | LocomotionIntent::JumpDown => {
                self.update_transfer(dt);
                return;
            }
            LocomotionIntent::Land => {
                self.land_timer -= dt;
                if self.land_timer <= 0.0 {
                    self.intent = LocomotionIntent::Idle;
                    self.physics.stop_horizontal();
                }
            }
            _ => {}
        }

        if self.physics.grounded
            && matches!(
                self.intent,
                LocomotionIntent::Walk | LocomotionIntent::Run | LocomotionIntent::Idle
            )
        {
            if self.maybe_start_monitor_jump() {
                return;
            }
            if is_on_window_top(self.last_support.as_ref()) && fastrand(0.0, 1.0) < 0.012 {
                let r = fastrand(0.0, 1.0);
                if r < 0.45 && self.try_peek_drop_from_window() {
                    return;
                }
                if r < 0.75 && self.try_hop_window(Some(fastrand(0.0, 1.0) < 0.5)) {
                    return;
                }
                if self.try_hop_window(None) {
                    return;
                }
            }
            if fastrand(0.0, 1.0) < 0.01 && self.try_climb_window_side() {
                return;
            }
        }

        self.physics.apply_gravity(dt);
        self.position = self.physics.integrate(self.position, dt);

        let foot = Vec2::new(
            self.position.x + PET_FRAME_WIDTH / 2.0,
            self.position.y + self.foot_offset,
        );
        let was_airborne =
            self.intent == LocomotionIntent::Fall || !self.physics.grounded;
        let max_drop = if self.physics.grounded { 10.0 } else { 64.0 };
        let support = find_support_under(foot, &self.surfaces, max_drop).cloned();
        self.last_support = support.clone();
        if let Some(support) = support {
            if self.physics.velocity.y >= -20.0 {
                self.position.y = support.y - self.foot_offset;
                self.physics.bounce_land();
                if was_airborne
                    && matches!(self.intent, LocomotionIntent::Fall | LocomotionIntent::Drag)
                {
                    self.begin_land();
                } else if self.intent == LocomotionIntent::Fall {
                    self.begin_land();
                }
            }
        } else if !self.physics.grounded {
            self.physics.leave_ground();
            if self.intent != LocomotionIntent::Land {
                self.enter_fall();
            }
        } else {
            self.physics.leave_ground();
            self.enter_fall();
        }

        self.clamp_to_current_display();
    }

    fn update_climb_window(&mut self, dt: f32) {
        self.special_timer -= dt;
        // Slower haul so climbWindow-0..2 reads as grab → pull → crest.
        let speed = 78.0;
        if self.position.y > self.climb_target_y {
            self.position.y -= speed * dt;
            self.physics.leave_ground();
            if self.special_timer <= 0.0 {
                self.enter_fall();
            }
            return;
        }
        self.position.y = self.climb_target_y;
        self.physics.bounce_land();
        self.begin_land();
    }

    fn update_cling_top(&mut self, dt: f32) {
        let Some(d) = self.current_display() else {
            self.enter_fall();
            return;
        };
        self.position.y = d.work_top - 2.0;
        self.physics.velocity = Vec2::new(self.physics.velocity.x * 0.9, 0.0);
        self.special_timer -= dt;
        if self.special_timer > 0.0 {
            return;
        }
        self.physics.throw(self.facing as f32 * 80.0, 220.0);
        self.enter_fall();
    }

    fn update_transfer(&mut self, dt: f32) {
        let Some(target) = self.transfer_target else {
            self.enter_fall();
            return;
        };
        let speed = if self.intent == LocomotionIntent::JumpUp {
            260.0
        } else {
            340.0
        };
        let dx = target.x - self.position.x;
        let dy = target.y - self.position.y;
        let dist = dx.hypot(dy);
        if dist < 6.0 {
            self.position = target;
            self.transfer_target = None;
            self.physics.bounce_land();
            self.begin_land();
            return;
        }
        let step = dist.min(speed * dt);
        self.position.x += (dx / dist) * step;
        self.position.y += (dy / dist) * step;
        self.physics.leave_ground();
    }

    fn maybe_start_monitor_jump(&mut self) -> bool {
        let Some(d) = self.current_display() else {
            return false;
        };
        let cx = self.position.x + PET_FRAME_WIDTH / 2.0;
        let near_left = cx <= d.left + SIDE_PAD + 40.0;
        let near_right = cx >= d.right - SIDE_PAD - 40.0;
        if !near_left && !near_right {
            return false;
        }
        let dir: i8 = if near_right { 1 } else { -1 };
        let Some(neighbor) = self.find_horizontal_neighbor(&d, dir) else {
            return false;
        };
        let floor_delta = neighbor.floor_y - d.floor_y;
        if floor_delta.abs() < 40.0 {
            return false;
        }
        if floor_delta < 0.0 && -floor_delta > MAX_JUMP_UP_PX {
            return false;
        }
        if floor_delta > 0.0 && floor_delta > MAX_JUMP_DOWN_PX {
            return false;
        }
        if fastrand(0.0, 1.0) > 0.35 && self.intent != LocomotionIntent::Run {
            return false;
        }
        let target_x = clamp(
            self.position.x + dir as f32 * 120.0,
            neighbor.left - SIDE_PAD + EDGE_INSET,
            neighbor.right - PET_FRAME_WIDTH + SIDE_PAD - EDGE_INSET,
        );
        self.facing = dir;
        self.physics.stop_horizontal();
        self.physics.leave_ground();
        self.transfer_target = Some(Vec2::new(target_x, neighbor.floor_y - self.foot_offset));
        self.intent = if floor_delta < 0.0 {
            LocomotionIntent::JumpUp
        } else {
            LocomotionIntent::JumpDown
        };
        self.special_timer = 2.0;
        true
    }

    fn begin_land(&mut self) {
        self.intent = LocomotionIntent::Land;
        self.land_timer = 0.28;
        self.just_landed = true;
        self.reset_fall_style();
    }

    fn enter_fall(&mut self) {
        if self.intent == LocomotionIntent::Fall {
            return;
        }
        self.intent = LocomotionIntent::Fall;
        self.fall_style = pick_fall_style(self.physics.velocity.y.abs(), None);
        self.fall_style_timer = fall_style_hold();
    }

    fn enter_fall_with(&mut self, style: FallStyle) {
        if self.intent == LocomotionIntent::Fall {
            return;
        }
        self.intent = LocomotionIntent::Fall;
        self.fall_style = style;
        self.fall_style_timer = fall_style_hold() + 0.35;
    }

    fn tick_fall_style(&mut self, dt: f32) {
        self.fall_style_timer -= dt;
        if self.fall_style_timer > 0.0 {
            return;
        }
        self.fall_style = pick_fall_style(self.physics.velocity.y.abs(), Some(self.fall_style));
        self.fall_style_timer = fall_style_hold();
    }

    fn reset_fall_style(&mut self) {
        self.fall_style = FallStyle::Spread;
        self.fall_style_timer = 0.0;
    }

    fn begin_cling_top(&mut self, display: DisplayInfo) {
        self.intent = LocomotionIntent::ClingTop;
        self.special_timer = 1.2 + fastrand(0.0, 1.4);
        self.physics.velocity = Vec2::default();
        self.physics.leave_ground();
        self.reset_fall_style();
        self.position.y = display.work_top - 2.0;
    }

    fn snap_wall_lean_x(&mut self, display: &DisplayInfo) {
        // wallLean cells are right-packed; facing flips the bitmap so paws meet the wall.
        if self.facing == 1 {
            self.position.x = display.right - PET_FRAME_WIDTH + 2.0;
        } else {
            self.position.x = display.left - 2.0;
        }
    }

    fn finish_wall_lean(&mut self) {
        // Turn away from the wall and step inward so centered walk/idle cells
        // don't look like a teleport, and so we don't immediately re-hit the edge.
        let wall_facing = self.facing;
        self.facing = if wall_facing == 1 { -1 } else { 1 };
        if let Some(d) = self.current_display() {
            let pull = SIDE_PAD + 40.0;
            let min_x = d.left - SIDE_PAD + EDGE_INSET;
            let max_x = d.right - PET_FRAME_WIDTH + SIDE_PAD - EDGE_INSET;
            if wall_facing == 1 {
                self.position.x = clamp(d.right - PET_FRAME_WIDTH - pull, min_x, max_x);
            } else {
                self.position.x = clamp(d.left + pull, min_x, max_x);
            }
        }
        self.intent = LocomotionIntent::Idle;
        self.physics.stop_horizontal();
        self.wall_lean_cooldown = 4.5 + fastrand(0.0, 2.0);
    }

    fn push_drag_sample(&mut self, x: f32, y: f32) {
        let t = self.now_ms;
        self.drag_samples.push(DragSample { t_ms: t, x, y });
        let cutoff = t.saturating_sub(150);
        while self.drag_samples.len() > 2 && self.drag_samples[0].t_ms < cutoff {
            self.drag_samples.remove(0);
        }
    }

    fn estimate_throw_velocity(&self) -> (f32, f32) {
        if self.drag_samples.len() < 2 {
            return (0.0, 120.0);
        }
        let last = *self.drag_samples.last().unwrap();
        let mut prev = self.drag_samples[0];
        for i in (0..self.drag_samples.len().saturating_sub(1)).rev() {
            let s = self.drag_samples[i];
            if last.t_ms.saturating_sub(s.t_ms) >= 40 {
                prev = s;
                break;
            }
        }
        let dt = ((last.t_ms.saturating_sub(prev.t_ms)) as f32 / 1000.0).max(0.016);
        let mut vx = clamp((last.x - prev.x) / dt, -700.0, 700.0);
        let mut vy = clamp((last.y - prev.y) / dt, -MAX_THROW_UP_SPEED, 900.0);
        if vx.hypot(vy) < 80.0 {
            vx *= 0.5;
            vy = vy.max(180.0);
        }
        (vx, vy)
    }

    fn current_display(&self) -> Option<DisplayInfo> {
        let cx = self.position.x + PET_FRAME_WIDTH / 2.0;
        let cy = self.position.y + PET_FRAME_HEIGHT / 2.0;
        self.displays
            .iter()
            .copied()
            .find(|d| cx >= d.left && cx < d.right && cy >= d.top && cy < d.bottom)
            .or_else(|| self.displays.first().copied())
    }

    fn find_horizontal_neighbor(&self, from: &DisplayInfo, dir: i8) -> Option<DisplayInfo> {
        let edge_x = if dir == 1 { from.right } else { from.left };
        let mut best: Option<DisplayInfo> = None;
        let mut best_gap = f32::INFINITY;
        for d in &self.displays {
            if d.id == from.id {
                continue;
            }
            let other_edge = if dir == 1 { d.left } else { d.right };
            let gap = if dir == 1 {
                other_edge - edge_x
            } else {
                edge_x - other_edge
            };
            if gap < -40.0 || gap > 80.0 {
                continue;
            }
            let overlap = from.bottom.min(d.bottom) - from.top.max(d.top);
            if overlap < 40.0 {
                continue;
            }
            if gap < best_gap {
                best_gap = gap;
                best = Some(*d);
            }
        }
        best
    }

    fn clamp_to_current_display(&mut self) {
        let Some(d) = self.current_display() else {
            return;
        };
        let min_pet_x = d.left - SIDE_PAD + EDGE_INSET;
        let max_pet_x = d.right - PET_FRAME_WIDTH + SIDE_PAD - EDGE_INSET;
        let next_x = clamp(self.position.x, min_pet_x, max_pet_x);

        if (next_x - self.position.x).abs() > f32::EPSILON
            && matches!(self.intent, LocomotionIntent::Walk | LocomotionIntent::Run)
        {
            if self.wall_lean_cooldown <= 0.0
                && fastrand(0.0, 1.0) < 0.25
                && self.physics.grounded
            {
                self.facing = if next_x <= min_pet_x + 1.0 { -1 } else { 1 };
                self.physics.stop_horizontal();
                self.intent = LocomotionIntent::WallLean;
                self.special_timer = 2.0 + fastrand(0.0, 2.5);
                self.snap_wall_lean_x(&d);
            } else {
                // Bounce away from the edge instead of walking into it again.
                self.facing = if next_x <= min_pet_x + 1.0 { 1 } else { -1 };
                let speed = if self.intent == LocomotionIntent::Run {
                    RUN_SPEED
                } else {
                    WALK_SPEED
                };
                self.physics.set_horizontal(self.facing as f32 * speed);
                self.position.x = next_x;
            }
        } else if (next_x - self.position.x).abs() > f32::EPSILON
            && self.intent == LocomotionIntent::Fall
        {
            self.physics.velocity.x *= -0.55;
            self.facing = if self.physics.velocity.x >= 0.0 { 1 } else { -1 };
            self.position.x = next_x;
        } else {
            self.position.x = next_x;
        }

        let max_pet_y = d.floor_y - self.foot_offset;
        let min_pet_y = d.work_top - 4.0;
        if self.position.y <= min_pet_y && self.physics.velocity.y <= 0.0 {
            self.position.y = min_pet_y;
            self.begin_cling_top(d);
            return;
        }
        if self.position.y > max_pet_y {
            self.position.y = max_pet_y;
            if self.physics.velocity.y >= 0.0 {
                self.physics.bounce_land();
                if matches!(self.intent, LocomotionIntent::Fall | LocomotionIntent::Drag) {
                    self.begin_land();
                }
            }
        }
    }
}

fn fall_style_hold() -> f32 {
    fastrand(0.35, 0.55)
}

fn pick_fall_style(speed: f32, avoid: Option<FallStyle>) -> FallStyle {
    let fast = (speed / 650.0).clamp(0.0, 1.0);
    let styles = [
        FallStyle::Spread,
        FallStyle::Tumble,
        FallStyle::Curl,
        FallStyle::Flail,
    ];
    let mut weights = [0.0_f32; 4];
    let mut total = 0.0;
    for (i, style) in styles.iter().enumerate() {
        if avoid == Some(*style) {
            continue;
        }
        let weight = match style {
            FallStyle::Spread => 2.4 - 1.6 * fast,
            FallStyle::Curl => 2.2 - 1.3 * fast,
            FallStyle::Tumble => 0.85 + 3.1 * fast,
            FallStyle::Flail => 0.7 + 3.4 * fast,
        };
        let weight = weight.max(0.05);
        weights[i] = weight;
        total += weight;
    }
    let mut roll = fastrand(0.0, total);
    for (i, weight) in weights.iter().enumerate() {
        if *weight <= 0.0 {
            continue;
        }
        roll -= *weight;
        if roll <= 0.0 {
            return styles[i];
        }
    }
    styles
        .into_iter()
        .find(|style| avoid != Some(*style))
        .unwrap_or(FallStyle::Spread)
}

fn fastrand(min: f32, span: f32) -> f32 {
    // Cheap LCG without extra deps — fine for pet AI.
    use std::cell::Cell;
    thread_local! {
        static STATE: Cell<u64> = Cell::new(0xC0FFEE);
    }
    STATE.with(|s| {
        let mut x = s.get();
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        s.set(x);
        min + span * ((x as f32) / (u64::MAX as f32))
    })
}
