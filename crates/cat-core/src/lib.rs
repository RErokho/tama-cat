//! Pure game/domain logic for Desktop Cat. No Win32, no rendering.

pub mod activity;
pub mod behavior;
pub mod config;
pub mod context;
pub mod math;
pub mod movement;
pub mod pet;
pub mod physics;
pub mod scenes;
pub mod surfaces;
pub mod types;
pub mod world;

pub use activity::UserActivitySnapshot;
pub use config::*;
pub use context::CompanionDirector;
pub use math::{clamp, Vec2};
pub use pet::{PetCore, PetPose};
pub use scenes::{SceneId, ScenePriority};
pub use types::*;
