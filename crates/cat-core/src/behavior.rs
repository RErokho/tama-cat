use crate::activity::UserActivitySnapshot;
use crate::context::CompanionDirector;
use crate::movement::{LocomotionIntent, MovementEngine};
use crate::types::{AnimClip, InternalNeeds};

#[derive(Debug, Clone)]
enum PlannedAction {
    Idle { seconds: f32 },
    Walk { direction: i8, seconds: f32 },
    Run { direction: i8, seconds: f32 },
    Sit { seconds: f32 },
    Sleep { seconds: f32 },
    Stretch { seconds: f32 },
}

pub struct BehaviorEngine {
    pub needs: InternalNeeds,
    pub companion: CompanionDirector,
    planner_queue: Vec<PlannedAction>,
    current: Option<PlannedAction>,
    elapsed: f32,
    decide_timer: f32,
    locked_facing: i8,
    body: AnimClip,
}

impl Default for BehaviorEngine {
    fn default() -> Self {
        Self::new(InternalNeeds::default())
    }
}

impl BehaviorEngine {
    pub fn new(needs: InternalNeeds) -> Self {
        Self {
            needs,
            companion: CompanionDirector::new(),
            planner_queue: Vec::new(),
            current: None,
            elapsed: 0.0,
            decide_timer: 0.5,
            locked_facing: 1,
            body: AnimClip::Idle,
        }
    }

    pub fn body(&self) -> AnimClip {
        self.body
    }

    pub fn set_body(&mut self, clip: AnimClip) {
        self.body = clip;
    }

    pub fn suspend_for_throw(&mut self) {
        self.suspend_locomotion_plan();
        self.companion.clear();
    }

    fn suspend_locomotion_plan(&mut self) {
        self.planner_queue.clear();
        self.current = None;
        self.elapsed = 0.0;
    }

    pub fn update(
        &mut self,
        dt: f32,
        movement: &mut MovementEngine,
        activity: &UserActivitySnapshot,
    ) {
        if movement.is_dragging() {
            self.body = AnimClip::Drag;
            return;
        }
        if movement.just_landed || movement.intent == LocomotionIntent::Land {
            self.suspend_locomotion_plan();
            self.body = AnimClip::Land;
            return;
        }
        if movement.airborne() || movement.special_pose() {
            self.suspend_locomotion_plan();
            self.body = movement.resolve_anim();
            return;
        }

        if matches!(
            self.body,
            AnimClip::WallLean
                | AnimClip::ClingTop
                | AnimClip::ClimbWindow
                | AnimClip::LookDown
                | AnimClip::HangDrop
                | AnimClip::JumpUp
                | AnimClip::JumpDown
        ) {
            self.locked_facing = movement.facing;
            self.decide_timer = self.decide_timer.max(0.9);
        }

        // Companion scenes (UserInput / Critical) beat idle planner.
        let scene = self.companion.update(dt, activity).copied();
        if let Some(scene) = scene {
            if scene.face_cursor {
                self.locked_facing = activity.cursor_dir_from_pet();
                movement.facing = self.locked_facing;
            }
            if scene.stop_movement {
                movement.stop();
                self.suspend_locomotion_plan();
            } else if matches!(scene.clip, AnimClip::Walk | AnimClip::Run) {
                let dir = activity.cursor_dir_from_pet();
                self.locked_facing = dir;
                if scene.clip == AnimClip::Run {
                    movement.start_run(dir);
                } else {
                    movement.start_walk(dir);
                }
            }
            self.body = scene.clip;
            self.decide_timer = self.decide_timer.max(0.4);
            return;
        }

        self.tick_planner(dt);
        if let Some(action) = self.current.clone() {
            self.apply_action(&action, movement);
            return;
        }

        self.decide_timer -= dt;
        if self.decide_timer <= 0.0 {
            let r = fastrand(0.0, 1.0);
            if r < 0.1 && movement.try_peek_drop_from_window() {
                self.decide_timer = 3.0;
                self.body = AnimClip::LookDown;
                return;
            }
            if r < 0.18 && movement.try_climb_window_side() {
                self.decide_timer = 3.0;
                self.body = AnimClip::ClimbWindow;
                return;
            }
            if r < 0.26 && movement.try_hop_window(Some(fastrand(0.0, 1.0) < 0.5)) {
                self.decide_timer = 2.5;
                self.body = movement.resolve_anim();
                return;
            }
            if r < 0.34 && movement.try_wall_lean() {
                self.decide_timer = 2.0;
                self.body = AnimClip::WallLean;
                return;
            }
            self.plan_next(movement);
            self.decide_timer = 1.5;
        }

        if movement.intent == LocomotionIntent::Idle && self.body != AnimClip::Sit {
            self.body = AnimClip::Idle;
        }
    }

    fn tick_planner(&mut self, dt: f32) {
        if self.current.is_none() {
            self.current = self.planner_queue.first().cloned();
            if !self.planner_queue.is_empty() {
                self.planner_queue.remove(0);
            }
            self.elapsed = 0.0;
        }
        let Some(action) = self.current.clone() else {
            return;
        };
        self.elapsed += dt;
        let seconds = match action {
            PlannedAction::Idle { seconds }
            | PlannedAction::Walk { seconds, .. }
            | PlannedAction::Run { seconds, .. }
            | PlannedAction::Sit { seconds }
            | PlannedAction::Sleep { seconds }
            | PlannedAction::Stretch { seconds } => seconds,
        };
        if self.elapsed >= seconds {
            self.current = None;
            self.elapsed = 0.0;
        }
    }

    fn plan_next(&mut self, movement: &MovementEngine) {
        if self.needs.sleepiness > 0.75 {
            self.enqueue(PlannedAction::Sit {
                seconds: 4.0 + fastrand(0.0, 4.0),
            });
            self.enqueue(PlannedAction::Sleep {
                seconds: 14.0 + fastrand(0.0, 20.0),
            });
            self.enqueue(PlannedAction::Idle {
                seconds: 3.0 + fastrand(0.0, 3.0),
            });
            self.needs.sleepiness = (self.needs.sleepiness - 0.5).max(0.0);
            return;
        }

        let roll = fastrand(0.0, 1.0);
        let keep_dir = fastrand(0.0, 1.0) < 0.7;
        let dir = if keep_dir {
            self.locked_facing
        } else if self.locked_facing == 1 {
            -1
        } else {
            1
        };
        self.locked_facing = dir;
        let _ = movement;

        if roll < 0.36 {
            self.enqueue(PlannedAction::Idle {
                seconds: 5.0 + fastrand(0.0, 8.0),
            });
        } else if roll < 0.55 {
            self.enqueue(PlannedAction::Sit {
                seconds: 10.0 + fastrand(0.0, 14.0),
            });
            self.enqueue(PlannedAction::Idle {
                seconds: 3.0 + fastrand(0.0, 4.0),
            });
        } else if roll < 0.68 {
            self.enqueue(PlannedAction::Stretch {
                seconds: 2.4 + fastrand(0.0, 1.2),
            });
            self.enqueue(PlannedAction::Idle {
                seconds: 2.0 + fastrand(0.0, 3.0),
            });
        } else if roll < 0.9 {
            self.enqueue(PlannedAction::Walk {
                direction: dir,
                seconds: 4.0 + fastrand(0.0, 6.0),
            });
            self.enqueue(PlannedAction::Idle {
                seconds: 2.0 + fastrand(0.0, 3.0),
            });
        } else {
            self.enqueue(PlannedAction::Run {
                direction: dir,
                seconds: 1.5 + fastrand(0.0, 2.0),
            });
            self.enqueue(PlannedAction::Idle {
                seconds: 2.0 + fastrand(0.0, 3.0),
            });
        }
    }

    fn apply_action(&mut self, action: &PlannedAction, movement: &mut MovementEngine) {
        match *action {
            PlannedAction::Walk { direction, .. } => {
                movement.start_walk(direction);
                self.locked_facing = direction;
                self.body = AnimClip::Walk;
            }
            PlannedAction::Run { direction, .. } => {
                movement.start_run(direction);
                self.locked_facing = direction;
                self.body = AnimClip::Run;
            }
            PlannedAction::Sit { .. } => {
                movement.stop();
                self.body = AnimClip::Sit;
            }
            PlannedAction::Stretch { .. } => {
                movement.stop();
                self.body = AnimClip::Stretch;
            }
            PlannedAction::Sleep { .. } => {
                movement.stop();
                self.body = AnimClip::Sleep;
            }
            PlannedAction::Idle { .. } => {
                movement.stop();
                self.body = AnimClip::Idle;
            }
        }
    }

    fn enqueue(&mut self, action: PlannedAction) {
        self.planner_queue.push(action);
    }
}

fn fastrand(min: f32, span: f32) -> f32 {
    use std::cell::Cell;
    thread_local! {
        static STATE: Cell<u64> = Cell::new(0xBEEF);
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
