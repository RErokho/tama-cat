# Desktop Cat — поведение (сжатое ТЗ)

Живой кот на рабочем столе. **Нет** голода, воды, магазина, прокачки, тамагочи.

## Режимы

Сон · Бодрствование · Активность · Игра · Взаимодействие · Компаньон (реакции на пользователя)

## Движки

Perception (`cat-platform::PerceptionSensor`) → `UserActivitySnapshot` → `CompanionDirector` / Context → `BehaviorEngine` → Movement + `AnimClip` → Render

Приоритеты: Critical > UserInput > Unexpected > CurrentGoal > Desire > Random

Правила сценок:
- `fire_chance` — шанс на **одну попытку** (не каждый кадр): continuous ~раз в 1.15с, edge-события — один бросок.
- Глобальный кулдаун ~3.5с между обычными сценками; Critical/edge могут прервать.
- Per-scene cooldown длиннее для частых триггеров (взгляд за курсором, печать).
- drag / полёт / specialPose всегда выше сценок.

## Клипы (sheet → AnimClip)

Базовые: idle, sit, walk, run, sleep, fall, land, drag, climb, groom, stretch, blink  
Эмоции / реакция: look, yawn, turn, alert, happy, annoyed, pounce  
Окна: clingTop, wallLean, jumpUp, jumpDown, lookDown, hangDrop, climbWindow  
Падение (`Fall`): клип чередуется на лету — `fall`, `fallTumble`, `fallCurl`, `fallFlail` (при большой |vy| чаще кувырок и паника). Приземление остаётся `land`.
Peek-drop с окна: `lookDown` (голова вниз к краю) → `hangDrop` (вис на передних лапах сверху, голова у хвата смотрит вверх к полке, тело свисает вниз, top-align) → fall.
Залаз по краю окна: `climbWindow` 3 фазы (хват → тяга → перевал), медленнее по Y.

Новые позы без отдельного арта пока мапятся на эти клипы (reuse-first).

## Каталог companion SceneId (62)

| Группа | SceneId |
| --- | --- |
| Печать | AttentiveReader, TypingHelperPaw, ConfusedTilt, BoredWatcher, EnterReact, BackspaceScare, TypingBreakGroom, StillThereBlink, CapsLockSurprise, CtrlSPurr, CtrlZJudgment, EscFlinch |
| Курсор | CursorHunt, GazeFollow, CursorSwipe, CursorGone, PawTapFloor, ScrollTrack, CursorLeap, RubCursor, CornerShutdownStare, DoubleClickChase |
| Текст / окна | ReadAlong, SelectionInterest, BigSelectionConfused, SleepOnDoc, NewWindowLook, WindowClosedLook, PeekBehindWindow, AltTabWatch, CopyCatch, PasteDrop, FileDragChase, PrintScreenPose |
| Активность | UserReturned, UserAwaySleep, SyncYawn, SuddenActivity, CheckOwner, SelfGroomBusy, HeardSomething, HappyGreeting, StayNearby, AskAttention, SlowBlinkTrust, BellyUp |
| Система | VirtualDesktopLook, SessionLockSleep, NightMood, ToastWatch, VolumeEars, MonitorTurn, ErrorHiss, SpinnerPaw |
| Характер | StudyPaws, TailChase, BigStretch, SuspiciousLook, FaceHideSleep, ExplorerWalk, StrangeSound, KneadSettle |

## MVP группы

- Idle / groom / sleep / walk-run / emotions
- Click / drag / companion cursor & typing reactions
- Multi-monitor, taskbar, window tops, jump/fall
- Habits memory, tray, pause, settings — позже

Исходники сценок: `crates/cat-core/src/scenes.rs`. Сенсоры: `crates/cat-platform/src/perception.rs`.
