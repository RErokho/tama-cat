//! Companion scene catalog: trigger → clip, priority, cooldown.

use crate::activity::UserActivitySnapshot;
use crate::types::AnimClip;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScenePriority {
    Random = 0,
    Desire = 1,
    CurrentGoal = 2,
    Unexpected = 3,
    UserInput = 4,
    Critical = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SceneId {
    // User canon 1–50
    AttentiveReader,
    TypingHelperPaw,
    ConfusedTilt,
    BoredWatcher,
    EnterReact,
    BackspaceScare,
    TypingBreakGroom,
    StillThereBlink,
    CursorHunt,
    GazeFollow,
    CursorSwipe,
    CursorGone,
    PawTapFloor,
    ScrollTrack,
    CursorLeap,
    ReadAlong,
    SelectionInterest,
    BigSelectionConfused,
    SleepOnDoc,
    NewWindowLook,
    WindowClosedLook,
    PeekBehindWindow,
    AltTabWatch,
    UserReturned,
    UserAwaySleep,
    SyncYawn,
    SuddenActivity,
    CheckOwner,
    SelfGroomBusy,
    HeardSomething,
    VirtualDesktopLook,
    SessionLockSleep,
    NightMood,
    ToastWatch,
    VolumeEars,
    MonitorTurn,
    StudyPaws,
    TailChase,
    BigStretch,
    SuspiciousLook,
    FaceHideSleep,
    ExplorerWalk,
    StrangeSound,
    KneadSettle,
    RubCursor,
    SlowBlinkTrust,
    BellyUp,
    AskAttention,
    HappyGreeting,
    StayNearby,
    // Lead extras
    CapsLockSurprise,
    CtrlSPurr,
    CtrlZJudgment,
    EscFlinch,
    CopyCatch,
    PasteDrop,
    FileDragChase,
    PrintScreenPose,
    ErrorHiss,
    SpinnerPaw,
    CornerShutdownStare,
    DoubleClickChase,
}

#[derive(Debug, Clone, Copy)]
pub struct SceneDef {
    pub id: SceneId,
    pub priority: ScenePriority,
    pub clip: AnimClip,
    pub duration: f32,
    pub cooldown: f32,
    pub face_cursor: bool,
    pub stop_movement: bool,
    pub fire_chance: f32,
}

impl SceneId {
    pub fn all() -> &'static [SceneId] {
        &ALL_SCENES
    }
}

const ALL_SCENES: [SceneId; 62] = [
    SceneId::AttentiveReader,
    SceneId::TypingHelperPaw,
    SceneId::ConfusedTilt,
    SceneId::BoredWatcher,
    SceneId::EnterReact,
    SceneId::BackspaceScare,
    SceneId::TypingBreakGroom,
    SceneId::StillThereBlink,
    SceneId::CursorHunt,
    SceneId::GazeFollow,
    SceneId::CursorSwipe,
    SceneId::CursorGone,
    SceneId::PawTapFloor,
    SceneId::ScrollTrack,
    SceneId::CursorLeap,
    SceneId::ReadAlong,
    SceneId::SelectionInterest,
    SceneId::BigSelectionConfused,
    SceneId::SleepOnDoc,
    SceneId::NewWindowLook,
    SceneId::WindowClosedLook,
    SceneId::PeekBehindWindow,
    SceneId::AltTabWatch,
    SceneId::UserReturned,
    SceneId::UserAwaySleep,
    SceneId::SyncYawn,
    SceneId::SuddenActivity,
    SceneId::CheckOwner,
    SceneId::SelfGroomBusy,
    SceneId::HeardSomething,
    SceneId::VirtualDesktopLook,
    SceneId::SessionLockSleep,
    SceneId::NightMood,
    SceneId::ToastWatch,
    SceneId::VolumeEars,
    SceneId::MonitorTurn,
    SceneId::StudyPaws,
    SceneId::TailChase,
    SceneId::BigStretch,
    SceneId::SuspiciousLook,
    SceneId::FaceHideSleep,
    SceneId::ExplorerWalk,
    SceneId::StrangeSound,
    SceneId::KneadSettle,
    SceneId::RubCursor,
    SceneId::SlowBlinkTrust,
    SceneId::BellyUp,
    SceneId::AskAttention,
    SceneId::HappyGreeting,
    SceneId::StayNearby,
    SceneId::CapsLockSurprise,
    SceneId::CtrlSPurr,
    SceneId::CtrlZJudgment,
    SceneId::EscFlinch,
    SceneId::CopyCatch,
    SceneId::PasteDrop,
    SceneId::FileDragChase,
    SceneId::PrintScreenPose,
    SceneId::ErrorHiss,
    SceneId::SpinnerPaw,
    SceneId::CornerShutdownStare,
    SceneId::DoubleClickChase,
];

pub fn def(id: SceneId) -> SceneDef {
    // fire_chance = вероятность на ОДНУ попытку (~1.15с для continuous, 1 раз на edge-событие).
    // Не крутится каждый кадр — иначе 0.5 ≈ всегда.
    match id {
        SceneId::AttentiveReader => SceneDef {
            id,
            priority: ScenePriority::UserInput,
            clip: AnimClip::Sit,
            duration: 5.5,
            cooldown: 18.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.22,
        },
        SceneId::TypingHelperPaw => SceneDef {
            id,
            priority: ScenePriority::Desire,
            clip: AnimClip::PawKeyboard,
            duration: 4.5,
            cooldown: 55.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.18,
        },
        SceneId::ConfusedTilt => SceneDef {
            id,
            priority: ScenePriority::UserInput,
            clip: AnimClip::Alert,
            duration: 3.0,
            cooldown: 22.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.20,
        },
        SceneId::BoredWatcher => SceneDef {
            id,
            priority: ScenePriority::Desire,
            clip: AnimClip::Yawn,
            duration: 4.0,
            cooldown: 50.0,
            face_cursor: false,
            stop_movement: true,
            fire_chance: 0.22,
        },
        SceneId::EnterReact => SceneDef {
            id,
            priority: ScenePriority::Unexpected,
            clip: AnimClip::Alert,
            duration: 2.4,
            cooldown: 14.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.35,
        },
        SceneId::BackspaceScare => SceneDef {
            id,
            priority: ScenePriority::Unexpected,
            clip: AnimClip::Annoyed,
            duration: 2.8,
            cooldown: 16.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.40,
        },
        SceneId::TypingBreakGroom => SceneDef {
            id,
            priority: ScenePriority::Desire,
            clip: AnimClip::Groom,
            duration: 5.0,
            cooldown: 35.0,
            face_cursor: false,
            stop_movement: true,
            fire_chance: 0.28,
        },
        SceneId::StillThereBlink => SceneDef {
            id,
            priority: ScenePriority::Desire,
            clip: AnimClip::SlowBlink,
            duration: 3.5,
            cooldown: 40.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.25,
        },
        SceneId::CursorHunt => SceneDef {
            id,
            priority: ScenePriority::UserInput,
            clip: AnimClip::Alert,
            duration: 3.2,
            cooldown: 16.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.18,
        },
        SceneId::GazeFollow => SceneDef {
            id,
            priority: ScenePriority::UserInput,
            clip: AnimClip::Turn,
            duration: 2.8,
            cooldown: 10.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.15,
        },
        SceneId::CursorSwipe => SceneDef {
            id,
            priority: ScenePriority::Unexpected,
            clip: AnimClip::Pounce,
            duration: 1.8,
            cooldown: 18.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.30,
        },
        SceneId::CursorGone => SceneDef {
            id,
            priority: ScenePriority::Unexpected,
            clip: AnimClip::Alert,
            duration: 3.0,
            cooldown: 25.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.40,
        },
        SceneId::PawTapFloor => SceneDef {
            id,
            priority: ScenePriority::Desire,
            clip: AnimClip::Annoyed,
            duration: 2.5,
            cooldown: 28.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.22,
        },
        SceneId::ScrollTrack => SceneDef {
            id,
            priority: ScenePriority::UserInput,
            clip: AnimClip::Look,
            duration: 3.0,
            cooldown: 12.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.20,
        },
        SceneId::CursorLeap => SceneDef {
            id,
            priority: ScenePriority::Unexpected,
            clip: AnimClip::Pounce,
            duration: 2.0,
            cooldown: 20.0,
            face_cursor: true,
            stop_movement: false,
            fire_chance: 0.28,
        },
        SceneId::ReadAlong => SceneDef {
            id,
            priority: ScenePriority::CurrentGoal,
            clip: AnimClip::Sit,
            duration: 7.0,
            cooldown: 35.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.25,
        },
        SceneId::SelectionInterest => SceneDef {
            id,
            priority: ScenePriority::UserInput,
            clip: AnimClip::Alert,
            duration: 3.5,
            cooldown: 18.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.30,
        },
        SceneId::BigSelectionConfused => SceneDef {
            id,
            priority: ScenePriority::Unexpected,
            clip: AnimClip::Annoyed,
            duration: 3.2,
            cooldown: 25.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.35,
        },
        SceneId::SleepOnDoc => SceneDef {
            id,
            priority: ScenePriority::Desire,
            clip: AnimClip::Sleep,
            duration: 14.0,
            cooldown: 80.0,
            face_cursor: false,
            stop_movement: true,
            fire_chance: 0.30,
        },
        SceneId::NewWindowLook => SceneDef {
            id,
            priority: ScenePriority::Unexpected,
            clip: AnimClip::Turn,
            duration: 3.0,
            cooldown: 14.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.40,
        },
        SceneId::WindowClosedLook => SceneDef {
            id,
            priority: ScenePriority::Unexpected,
            clip: AnimClip::Alert,
            duration: 2.8,
            cooldown: 14.0,
            face_cursor: false,
            stop_movement: true,
            fire_chance: 0.35,
        },
        SceneId::PeekBehindWindow => SceneDef {
            id,
            priority: ScenePriority::Desire,
            clip: AnimClip::LookDown,
            duration: 3.5,
            cooldown: 45.0,
            face_cursor: false,
            stop_movement: true,
            fire_chance: 0.15,
        },
        SceneId::AltTabWatch => SceneDef {
            id,
            priority: ScenePriority::UserInput,
            clip: AnimClip::Turn,
            duration: 2.8,
            cooldown: 12.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.35,
        },
        SceneId::UserReturned => SceneDef {
            id,
            priority: ScenePriority::Critical,
            clip: AnimClip::Stretch,
            duration: 4.0,
            cooldown: 45.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.90,
        },
        SceneId::UserAwaySleep => SceneDef {
            id,
            priority: ScenePriority::CurrentGoal,
            clip: AnimClip::Sleep,
            duration: 18.0,
            cooldown: 55.0,
            face_cursor: false,
            stop_movement: true,
            fire_chance: 0.45,
        },
        SceneId::SyncYawn => SceneDef {
            id,
            priority: ScenePriority::Desire,
            clip: AnimClip::Yawn,
            duration: 3.5,
            cooldown: 60.0,
            face_cursor: false,
            stop_movement: true,
            fire_chance: 0.20,
        },
        SceneId::SuddenActivity => SceneDef {
            id,
            priority: ScenePriority::Unexpected,
            clip: AnimClip::Alert,
            duration: 3.0,
            cooldown: 30.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.45,
        },
        SceneId::CheckOwner => SceneDef {
            id,
            priority: ScenePriority::Desire,
            clip: AnimClip::SlowBlink,
            duration: 3.5,
            cooldown: 55.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.20,
        },
        SceneId::SelfGroomBusy => SceneDef {
            id,
            priority: ScenePriority::Random,
            clip: AnimClip::Groom,
            duration: 5.5,
            cooldown: 40.0,
            face_cursor: false,
            stop_movement: true,
            fire_chance: 0.22,
        },
        SceneId::HeardSomething => SceneDef {
            id,
            priority: ScenePriority::Unexpected,
            clip: AnimClip::Alert,
            duration: 2.8,
            cooldown: 28.0,
            face_cursor: false,
            stop_movement: true,
            fire_chance: 0.25,
        },
        SceneId::VirtualDesktopLook => SceneDef {
            id,
            priority: ScenePriority::Unexpected,
            clip: AnimClip::Turn,
            duration: 3.5,
            cooldown: 20.0,
            face_cursor: false,
            stop_movement: true,
            fire_chance: 0.55,
        },
        SceneId::SessionLockSleep => SceneDef {
            id,
            priority: ScenePriority::Critical,
            clip: AnimClip::Sleep,
            duration: 30.0,
            cooldown: 8.0,
            face_cursor: false,
            stop_movement: true,
            fire_chance: 1.0,
        },
        SceneId::NightMood => SceneDef {
            id,
            priority: ScenePriority::Desire,
            clip: AnimClip::Sit,
            duration: 8.0,
            cooldown: 150.0,
            face_cursor: false,
            stop_movement: true,
            fire_chance: 0.20,
        },
        SceneId::ToastWatch => SceneDef {
            id,
            priority: ScenePriority::Unexpected,
            clip: AnimClip::Alert,
            duration: 3.0,
            cooldown: 18.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.40,
        },
        SceneId::VolumeEars => SceneDef {
            id,
            priority: ScenePriority::UserInput,
            clip: AnimClip::Alert,
            duration: 2.5,
            cooldown: 12.0,
            face_cursor: false,
            stop_movement: true,
            fire_chance: 0.45,
        },
        SceneId::MonitorTurn => SceneDef {
            id,
            priority: ScenePriority::UserInput,
            clip: AnimClip::Turn,
            duration: 3.0,
            cooldown: 16.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.45,
        },
        SceneId::StudyPaws => SceneDef {
            id,
            priority: ScenePriority::Random,
            clip: AnimClip::Groom,
            duration: 4.5,
            cooldown: 50.0,
            face_cursor: false,
            stop_movement: true,
            fire_chance: 0.18,
        },
        SceneId::TailChase => SceneDef {
            id,
            priority: ScenePriority::Random,
            clip: AnimClip::TailChase,
            duration: 3.0,
            cooldown: 70.0,
            face_cursor: false,
            stop_movement: true,
            fire_chance: 0.12,
        },
        SceneId::BigStretch => SceneDef {
            id,
            priority: ScenePriority::Desire,
            clip: AnimClip::Stretch,
            duration: 4.0,
            cooldown: 45.0,
            face_cursor: false,
            stop_movement: true,
            fire_chance: 0.22,
        },
        SceneId::SuspiciousLook => SceneDef {
            id,
            priority: ScenePriority::Random,
            clip: AnimClip::Turn,
            duration: 3.0,
            cooldown: 55.0,
            face_cursor: false,
            stop_movement: true,
            fire_chance: 0.15,
        },
        SceneId::FaceHideSleep => SceneDef {
            id,
            priority: ScenePriority::Desire,
            clip: AnimClip::FaceHide,
            duration: 16.0,
            cooldown: 90.0,
            face_cursor: false,
            stop_movement: true,
            fire_chance: 0.25,
        },
        SceneId::ExplorerWalk => SceneDef {
            id,
            priority: ScenePriority::Random,
            clip: AnimClip::Walk,
            duration: 7.0,
            cooldown: 40.0,
            face_cursor: false,
            stop_movement: false,
            fire_chance: 0.20,
        },
        SceneId::StrangeSound => SceneDef {
            id,
            priority: ScenePriority::Unexpected,
            clip: AnimClip::Alert,
            duration: 3.0,
            cooldown: 50.0,
            face_cursor: false,
            stop_movement: true,
            fire_chance: 0.12,
        },
        SceneId::KneadSettle => SceneDef {
            id,
            priority: ScenePriority::Desire,
            clip: AnimClip::Knead,
            duration: 5.5,
            cooldown: 60.0,
            face_cursor: false,
            stop_movement: true,
            fire_chance: 0.18,
        },
        SceneId::RubCursor => SceneDef {
            id,
            priority: ScenePriority::UserInput,
            clip: AnimClip::Rub,
            duration: 3.5,
            cooldown: 14.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.35,
        },
        SceneId::SlowBlinkTrust => SceneDef {
            id,
            priority: ScenePriority::Desire,
            clip: AnimClip::SlowBlink,
            duration: 4.0,
            cooldown: 28.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.28,
        },
        SceneId::BellyUp => SceneDef {
            id,
            priority: ScenePriority::Desire,
            clip: AnimClip::BellyUp,
            duration: 4.5,
            cooldown: 40.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.22,
        },
        SceneId::AskAttention => SceneDef {
            id,
            priority: ScenePriority::Desire,
            clip: AnimClip::AttentionPaw,
            duration: 4.0,
            cooldown: 70.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.25,
        },
        SceneId::HappyGreeting => SceneDef {
            id,
            priority: ScenePriority::Critical,
            clip: AnimClip::Happy,
            duration: 4.5,
            cooldown: 90.0,
            face_cursor: true,
            stop_movement: false,
            fire_chance: 0.85,
        },
        SceneId::StayNearby => SceneDef {
            id,
            priority: ScenePriority::CurrentGoal,
            clip: AnimClip::Sit,
            duration: 8.0,
            cooldown: 40.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.18,
        },
        SceneId::CapsLockSurprise => SceneDef {
            id,
            priority: ScenePriority::Unexpected,
            clip: AnimClip::Alert,
            duration: 2.8,
            cooldown: 30.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.45,
        },
        SceneId::CtrlSPurr => SceneDef {
            id,
            priority: ScenePriority::UserInput,
            clip: AnimClip::Happy,
            duration: 3.0,
            cooldown: 25.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.35,
        },
        SceneId::CtrlZJudgment => SceneDef {
            id,
            priority: ScenePriority::UserInput,
            clip: AnimClip::Annoyed,
            duration: 2.8,
            cooldown: 20.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.35,
        },
        SceneId::EscFlinch => SceneDef {
            id,
            priority: ScenePriority::Unexpected,
            clip: AnimClip::Annoyed,
            duration: 2.2,
            cooldown: 16.0,
            face_cursor: false,
            stop_movement: true,
            fire_chance: 0.30,
        },
        SceneId::CopyCatch => SceneDef {
            id,
            priority: ScenePriority::Unexpected,
            clip: AnimClip::Pounce,
            duration: 2.2,
            cooldown: 18.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.30,
        },
        SceneId::PasteDrop => SceneDef {
            id,
            priority: ScenePriority::Unexpected,
            clip: AnimClip::Alert,
            duration: 2.2,
            cooldown: 18.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.28,
        },
        SceneId::FileDragChase => SceneDef {
            id,
            priority: ScenePriority::UserInput,
            clip: AnimClip::Run,
            duration: 3.5,
            cooldown: 16.0,
            face_cursor: true,
            stop_movement: false,
            fire_chance: 0.40,
        },
        SceneId::PrintScreenPose => SceneDef {
            id,
            priority: ScenePriority::Unexpected,
            clip: AnimClip::Happy,
            duration: 3.0,
            cooldown: 25.0,
            face_cursor: false,
            stop_movement: true,
            fire_chance: 0.50,
        },
        SceneId::ErrorHiss => SceneDef {
            id,
            priority: ScenePriority::Unexpected,
            clip: AnimClip::Annoyed,
            duration: 3.0,
            cooldown: 35.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.35,
        },
        SceneId::SpinnerPaw => SceneDef {
            id,
            priority: ScenePriority::Desire,
            clip: AnimClip::Groom,
            duration: 4.0,
            cooldown: 40.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.20,
        },
        SceneId::CornerShutdownStare => SceneDef {
            id,
            priority: ScenePriority::Unexpected,
            clip: AnimClip::Alert,
            duration: 3.5,
            cooldown: 30.0,
            face_cursor: true,
            stop_movement: true,
            fire_chance: 0.40,
        },
        SceneId::DoubleClickChase => SceneDef {
            id,
            priority: ScenePriority::UserInput,
            clip: AnimClip::Run,
            duration: 5.0,
            cooldown: 15.0,
            face_cursor: true,
            stop_movement: false,
            fire_chance: 1.0,
        },
    }
}

/// Discrete one-shot user/OS events (roll once, not every pick interval).
pub fn is_edge_trigger(activity: &UserActivitySnapshot) -> bool {
    activity.enter_pressed
        || activity.backspace_burst
        || activity.caps_lock_toggled
        || activity.ctrl_s
        || activity.ctrl_z
        || activity.esc_pressed
        || activity.clipboard_changed
        || activity.print_screen
        || activity.left_double_click_pet
        || activity.session_unlocked
        || activity.returned_from_idle
        || activity.window_opened
        || activity.window_closed
        || activity.foreground_changed
        || activity.virtual_desktop_changed
        || activity.volume_changed
        || activity.monitor_changed
        || activity.toast_likely
        || activity.cursor_offscreen
        || activity.cursor_in_corner
}

/// Which scenes are candidates given the current snapshot + soft context flags.
pub fn candidates(activity: &UserActivitySnapshot, long_typing_secs: f32, near_secs: f32) -> Vec<SceneId> {
    let mut out = Vec::new();

    // --- Critical / away ---
    if activity.session_locked {
        out.push(SceneId::SessionLockSleep);
        return out;
    }
    if activity.session_unlocked {
        out.push(SceneId::HappyGreeting);
    }
    if activity.returned_from_idle {
        out.push(SceneId::UserReturned);
        // SuddenActivity only after longer AFK (proxy: deep idle already folded into unlock)
        if activity.idle_secs < 1.0 {
            out.push(SceneId::SuddenActivity);
        }
    }

    // --- Typing ---
    if activity.typing {
        // Sit and watch after a short settle, not from the first key.
        if long_typing_secs > 2.5 {
            out.push(SceneId::AttentiveReader);
        }
        if long_typing_secs > 12.0 {
            out.push(SceneId::StayNearby);
        }
        if long_typing_secs > 35.0 {
            out.push(SceneId::TypingHelperPaw);
            out.push(SceneId::BoredWatcher);
            out.push(SceneId::SyncYawn);
        }
        if activity.typing_rate > 10.0 && long_typing_secs > 4.0 {
            out.push(SceneId::ConfusedTilt);
        }
    }
    if activity.enter_pressed {
        out.push(SceneId::EnterReact);
    }
    if activity.backspace_burst {
        out.push(SceneId::BackspaceScare);
    }
    // Pause after typing: groom / slow blink
    if !activity.typing
        && (5.0..25.0).contains(&activity.idle_secs)
        && long_typing_secs > 4.0
    {
        out.push(SceneId::TypingBreakGroom);
        out.push(SceneId::StillThereBlink);
    }

    // --- Cursor ---
    if activity.cursor_speed > 80.0 && activity.cursor_speed < 700.0 {
        out.push(SceneId::GazeFollow);
    }
    if activity.cursor_speed > 180.0 && activity.cursor_speed < 700.0 {
        out.push(SceneId::CursorHunt);
    }
    if activity.cursor_speed > 1100.0 {
        out.push(SceneId::CursorLeap);
        out.push(SceneId::PawTapFloor);
    }
    // Near pet: rub / trust / belly / swipe
    if activity.cursor_near_pet && activity.cursor_speed < 90.0 {
        if near_secs > 1.2 {
            out.push(SceneId::RubCursor);
        }
        if near_secs > 3.0 {
            out.push(SceneId::SlowBlinkTrust);
        }
        if near_secs > 6.0 {
            out.push(SceneId::BellyUp);
        }
        if near_secs > 1.5 && activity.cursor_speed < 25.0 {
            out.push(SceneId::CursorSwipe);
        }
    }
    if activity.cursor_offscreen {
        out.push(SceneId::CursorGone);
    }
    if activity.scroll_delta.abs() > 1.0 {
        out.push(SceneId::ScrollTrack);
    }
    if activity.cursor_in_corner {
        out.push(SceneId::CornerShutdownStare);
    }

    // --- Selection / reading ---
    if activity.selection_likely && !activity.typing {
        out.push(SceneId::SelectionInterest);
        if activity.idle_secs > 2.0 {
            out.push(SceneId::ReadAlong);
        }
    }
    if activity.selection_large {
        out.push(SceneId::BigSelectionConfused);
    }

    // --- Away / sleep ---
    if activity.idle_secs > 55.0 && !activity.typing {
        out.push(SceneId::UserAwaySleep);
    }
    if activity.idle_secs > 75.0 && !activity.typing {
        out.push(SceneId::FaceHideSleep);
    }
    if activity.idle_secs > 110.0 && !activity.typing {
        out.push(SceneId::SleepOnDoc);
    }

    // --- Windows / system edges ---
    if activity.window_opened {
        out.push(SceneId::NewWindowLook);
    }
    if activity.window_closed {
        out.push(SceneId::WindowClosedLook);
    }
    if activity.foreground_changed {
        out.push(SceneId::AltTabWatch);
    }
    if activity.virtual_desktop_changed {
        out.push(SceneId::VirtualDesktopLook);
    }
    if activity.volume_changed {
        out.push(SceneId::VolumeEars);
    }
    if activity.toast_likely {
        out.push(SceneId::ToastWatch);
    }
    if activity.monitor_changed {
        out.push(SceneId::MonitorTurn);
    }
    if activity.caps_lock_toggled {
        out.push(SceneId::CapsLockSurprise);
    }
    if activity.ctrl_s {
        out.push(SceneId::CtrlSPurr);
    }
    if activity.ctrl_z {
        out.push(SceneId::CtrlZJudgment);
    }
    if activity.esc_pressed {
        out.push(SceneId::EscFlinch);
    }
    if activity.clipboard_changed {
        // Prefer catch over paste randomly via separate chances in def
        out.push(SceneId::CopyCatch);
        out.push(SceneId::PasteDrop);
    }
    if activity.file_drag_likely {
        out.push(SceneId::FileDragChase);
    }
    if activity.print_screen {
        out.push(SceneId::PrintScreenPose);
    }
    if activity.error_ui_likely {
        out.push(SceneId::ErrorHiss);
    }
    if activity.loading_likely {
        out.push(SceneId::SpinnerPaw);
    }
    if activity.left_double_click_pet {
        out.push(SceneId::DoubleClickChase);
    }
    if (activity.hour_of_day >= 22 || activity.hour_of_day < 6) && activity.idle_secs > 15.0 {
        out.push(SceneId::NightMood);
    }

    // --- Ambient character: calm only, few options (not a dump of 12) ---
    if activity.idle_secs > 14.0 && activity.cursor_speed < 15.0 && !activity.typing && !activity.cursor_near_pet
    {
        // One "family" per band of idle so it feels intentional.
        if activity.idle_secs < 40.0 {
            out.push(SceneId::BigStretch);
            out.push(SceneId::SelfGroomBusy);
            out.push(SceneId::CheckOwner);
        } else if activity.idle_secs < 80.0 {
            out.push(SceneId::StudyPaws);
            out.push(SceneId::KneadSettle);
            out.push(SceneId::ExplorerWalk);
            out.push(SceneId::AskAttention);
        } else {
            out.push(SceneId::SuspiciousLook);
            out.push(SceneId::TailChase);
            out.push(SceneId::StrangeSound);
            out.push(SceneId::PeekBehindWindow);
        }
    }

    out
}
