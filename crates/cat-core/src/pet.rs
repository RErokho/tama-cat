use crate::activity::UserActivitySnapshot;
use crate::behavior::BehaviorEngine;
use crate::config::{PET_FRAME_HEIGHT, PET_FRAME_WIDTH, TICK_HZ};
use crate::math::Vec2;
use crate::movement::MovementEngine;
use crate::types::{AnimClip, DisplayInfo, InternalNeeds, Surface};
use crate::world::WorldSystem;

#[derive(Debug, Clone, Copy)]
pub struct PetPose {
    pub x: f32,
    pub y: f32,
    pub facing: i8,
    pub anim: AnimClip,
    pub scale: f32,
    pub paused: bool,
}

pub struct PetCore {
    pub world: WorldSystem,
    pub movement: MovementEngine,
    pub behavior: BehaviorEngine,
    paused: bool,
    scale: f32,
    last_ms: u64,
}

impl PetCore {
    pub fn new() -> Self {
        Self {
            world: WorldSystem::new(),
            movement: MovementEngine::new(),
            behavior: BehaviorEngine::new(InternalNeeds::default()),
            paused: false,
            scale: 1.0,
            last_ms: 0,
        }
    }

    pub fn snap_visible(&mut self, displays: &[DisplayInfo]) {
        if let Some(d) = displays.first() {
            self.movement.snap_to_primary(d);
        }
    }

    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
    }

    pub fn pose(&self) -> PetPose {
        let drag_boost = if self.behavior.body() == AnimClip::Drag {
            1.15
        } else {
            1.0
        };
        PetPose {
            x: self.movement.position.x,
            y: self.movement.position.y,
            facing: self.movement.facing,
            anim: self.behavior.body(),
            scale: self.scale * drag_boost,
            paused: self.paused,
        }
    }

    pub fn begin_drag(&mut self) {
        self.behavior.suspend_for_throw();
        self.movement.begin_drag();
        self.behavior.set_body(AnimClip::Drag);
    }

    pub fn drag_to(&mut self, screen: Vec2) {
        self.movement.drag_to(screen);
    }

    pub fn end_drag(&mut self) {
        self.movement.end_drag();
        self.behavior.suspend_for_throw();
        self.behavior.set_body(self.movement.resolve_anim());
    }

    pub fn frame_size(&self) -> (u32, u32) {
        let pose = self.pose();
        (
            (PET_FRAME_WIDTH * pose.scale).round().max(1.0) as u32,
            (PET_FRAME_HEIGHT * pose.scale).round().max(1.0) as u32,
        )
    }

    /// One simulation step. Caller supplies displays, window surfaces, and activity.
    pub fn tick(
        &mut self,
        now_ms: u64,
        displays: Vec<DisplayInfo>,
        window_surfaces: Vec<Surface>,
        mut activity: UserActivitySnapshot,
    ) -> PetPose {
        let dt = if self.last_ms == 0 {
            1.0 / TICK_HZ
        } else {
            ((now_ms.saturating_sub(self.last_ms)) as f32 / 1000.0).min(0.05)
        };
        self.last_ms = now_ms;
        self.movement.set_clock_ms(now_ms);
        self.movement.set_displays(displays.clone());
        self.world.update(dt, displays, window_surfaces);

        activity.pet_x = self.movement.position.x + PET_FRAME_WIDTH * 0.5;
        activity.pet_y = self.movement.position.y + PET_FRAME_HEIGHT * 0.5;
        let near = (activity.cursor_x - activity.pet_x).hypot(activity.cursor_y - activity.pet_y);
        activity.cursor_near_pet = near < 120.0;

        if !self.paused {
            self.movement.update(dt, self.world.surfaces());
            self.behavior.update(dt, &mut self.movement, &activity);
        }

        self.pose()
    }
}

impl Default for PetCore {
    fn default() -> Self {
        Self::new()
    }
}
