//! Context detector + companion scene director (priorities, cooldowns, anti-spam).

use crate::activity::UserActivitySnapshot;
use crate::scenes::{candidates, def, is_edge_trigger, SceneDef, SceneId, ScenePriority};
use crate::types::AnimClip;
use std::collections::HashMap;

/// How often we roll continuous/ambient scenes (not every frame).
const PICK_INTERVAL_SECS: f32 = 1.15;
/// Minimum gap between non-critical scenes so the cat is not constantly reacting.
const GLOBAL_COOLDOWN_SECS: f32 = 3.5;

#[derive(Debug, Clone, Copy)]
pub struct ActiveScene {
    pub id: SceneId,
    pub clip: AnimClip,
    pub elapsed: f32,
    pub duration: f32,
    pub face_cursor: bool,
    pub stop_movement: bool,
    pub priority: ScenePriority,
}

pub struct CompanionDirector {
    cooldowns: HashMap<u32, f32>,
    pub active: Option<ActiveScene>,
    long_typing_secs: f32,
    near_cursor_secs: f32,
    pick_accum: f32,
    global_cool: f32,
    rng: u64,
}

impl Default for CompanionDirector {
    fn default() -> Self {
        Self::new()
    }
}

impl CompanionDirector {
    pub fn new() -> Self {
        Self {
            cooldowns: HashMap::new(),
            active: None,
            long_typing_secs: 0.0,
            near_cursor_secs: 0.0,
            pick_accum: 0.0,
            global_cool: 0.0,
            rng: 0xC0FFEE,
        }
    }

    fn scene_key(id: SceneId) -> u32 {
        id as u32
    }

    fn roll(&mut self) -> f32 {
        self.rng ^= self.rng << 13;
        self.rng ^= self.rng >> 7;
        self.rng ^= self.rng << 17;
        (self.rng as f32) / (u64::MAX as f32)
    }

    pub fn update(&mut self, dt: f32, activity: &UserActivitySnapshot) -> Option<&ActiveScene> {
        for v in self.cooldowns.values_mut() {
            *v = (*v - dt).max(0.0);
        }
        self.global_cool = (self.global_cool - dt).max(0.0);

        if activity.typing {
            self.long_typing_secs += dt;
        } else {
            self.long_typing_secs = (self.long_typing_secs - dt * 2.0).max(0.0);
        }

        if activity.cursor_near_pet {
            self.near_cursor_secs += dt;
        } else {
            self.near_cursor_secs = 0.0;
        }

        if let Some(active) = self.active.as_mut() {
            active.elapsed += dt;
            let done = active.elapsed >= active.duration;
            let preempt_lock =
                activity.session_locked && active.id != SceneId::SessionLockSleep;
            if done || preempt_lock {
                let finished = active.id;
                let cool = def(finished).cooldown;
                self.cooldowns.insert(Self::scene_key(finished), cool);
                self.active = None;
                self.global_cool = GLOBAL_COOLDOWN_SECS;
                self.pick_accum = 0.0;
            }
        }

        let edge = is_edge_trigger(activity);
        self.pick_accum += dt;
        let try_pick = edge || self.pick_accum >= PICK_INTERVAL_SECS;
        if try_pick {
            self.pick_accum = 0.0;
            if let Some(next) = self.pick(activity, edge) {
                let can_start = match &self.active {
                    None => true,
                    Some(cur) => (next.priority as u8) > (cur.priority as u8),
                };
                // Soft scenes wait for global cool; edges/critical may interrupt.
                let cool_ok = self.global_cool <= 0.0
                    || edge
                    || next.priority as u8 >= ScenePriority::Unexpected as u8;
                if can_start && cool_ok {
                    if let Some(cur) = self.active {
                        self.cooldowns
                            .insert(Self::scene_key(cur.id), def(cur.id).cooldown * 0.5);
                    }
                    self.active = Some(ActiveScene {
                        id: next.id,
                        clip: next.clip,
                        elapsed: 0.0,
                        duration: next.duration,
                        face_cursor: next.face_cursor,
                        stop_movement: next.stop_movement,
                        priority: next.priority,
                    });
                    self.cooldowns
                        .insert(Self::scene_key(next.id), next.cooldown);
                    if (next.priority as u8) < ScenePriority::Critical as u8 {
                        self.global_cool = GLOBAL_COOLDOWN_SECS;
                    }
                }
            }
        }

        self.active.as_ref()
    }

    fn pick(&mut self, activity: &UserActivitySnapshot, edge: bool) -> Option<SceneDef> {
        let mut best: Option<SceneDef> = None;
        let ids = candidates(activity, self.long_typing_secs, self.near_cursor_secs);
        for id in ids {
            let cd = self
                .cooldowns
                .get(&Self::scene_key(id))
                .copied()
                .unwrap_or(0.0);
            if cd > 0.0 {
                continue;
            }
            if self.active.map(|a| a.id) == Some(id) {
                continue;
            }
            let d = def(id);
            // Continuous scenes: one roll per pick interval. Edge scenes: one roll per event.
            if self.roll() > d.fire_chance {
                continue;
            }
            // Prefer edge-relevant reactions when an edge just fired.
            if edge && (d.priority as u8) < ScenePriority::Unexpected as u8 {
                continue;
            }
            match best {
                None => best = Some(d),
                Some(b) if (d.priority as u8) > (b.priority as u8) => best = Some(d),
                Some(b) if d.priority == b.priority && self.roll() < 0.4 => best = Some(d),
                _ => {}
            }
        }
        best
    }

    pub fn clear(&mut self) {
        self.active = None;
    }
}
