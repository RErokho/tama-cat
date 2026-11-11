use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum AnimClip {
    Idle,
    Sit,
    Walk,
    Run,
    Sleep,
    Fall,
    FallTumble,
    FallCurl,
    FallFlail,
    Land,
    Drag,
    Climb,
    Groom,
    Stretch,
    Blink,
    Look,
    Yawn,
    Turn,
    Alert,
    Happy,
    Annoyed,
    Pounce,
    Rub,
    PawKeyboard,
    BellyUp,
    SlowBlink,
    Knead,
    AttentionPaw,
    FaceHide,
    TailChase,
    ClingTop,
    WallLean,
    JumpUp,
    JumpDown,
    LookDown,
    HangDrop,
    ClimbWindow,
}

impl AnimClip {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Sit => "sit",
            Self::Walk => "walk",
            Self::Run => "run",
            Self::Sleep => "sleep",
            Self::Fall => "fall",
            Self::FallTumble => "fallTumble",
            Self::FallCurl => "fallCurl",
            Self::FallFlail => "fallFlail",
            Self::Land => "land",
            Self::Drag => "drag",
            Self::Climb => "climb",
            Self::Groom => "groom",
            Self::Stretch => "stretch",
            Self::Blink => "blink",
            Self::Look => "look",
            Self::Yawn => "yawn",
            Self::Turn => "turn",
            Self::Alert => "alert",
            Self::Happy => "happy",
            Self::Annoyed => "annoyed",
            Self::Pounce => "pounce",
            Self::Rub => "rub",
            Self::PawKeyboard => "pawKeyboard",
            Self::BellyUp => "bellyUp",
            Self::SlowBlink => "slowBlink",
            Self::Knead => "knead",
            Self::AttentionPaw => "attentionPaw",
            Self::FaceHide => "faceHide",
            Self::TailChase => "tailChase",
            Self::ClingTop => "clingTop",
            Self::WallLean => "wallLean",
            Self::JumpUp => "jumpUp",
            Self::JumpDown => "jumpDown",
            Self::LookDown => "lookDown",
            Self::HangDrop => "hangDrop",
            Self::ClimbWindow => "climbWindow",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SurfaceKind {
    Floor,
    Taskbar,
    WindowTop,
    WindowSide,
    ScreenEdge,
}

#[derive(Debug, Clone)]
pub struct Surface {
    pub id: String,
    pub kind: SurfaceKind,
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct DisplayInfo {
    pub id: u32,
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
    pub floor_y: f32,
    pub work_top: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct InternalNeeds {
    pub sleepiness: f32,
    pub curiosity: f32,
    pub boredom: f32,
    pub excitement: f32,
    pub mood: f32,
}

impl Default for InternalNeeds {
    fn default() -> Self {
        Self {
            sleepiness: 0.2,
            curiosity: 0.5,
            boredom: 0.3,
            excitement: 0.2,
            mood: 0.6,
        }
    }
}
