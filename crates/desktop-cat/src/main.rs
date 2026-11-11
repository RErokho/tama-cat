//! Desktop Cat — Rust entrypoint.

use cat_core::{PetCore, Vec2};
use cat_platform::{
    list_displays, list_window_surfaces, now_ms, PerceptionSensor, PetWindow, WindowEvent,
};
use cat_render::{SpritePlayer, SpriteSheet};
use std::path::{Path, PathBuf};
use std::time::Instant;

fn main() {
    if let Err(err) = run() {
        eprintln!("Desktop Cat failed: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let assets = resolve_assets_dir()?;
    let sheet_png = assets.join("sheet.png");
    let sheet_json = assets.join("sheet.json");
    if !sheet_png.exists() || !sheet_json.exists() {
        return Err(format!(
            "sprite sheet not found under {}",
            assets.display()
        )
        .into());
    }

    let sheet = SpriteSheet::load(&sheet_png, &sheet_json)?;
    let mut player = SpritePlayer::new(sheet);
    let mut pet = PetCore::new();
    let mut perception = PerceptionSensor::new();
    let displays = list_displays();
    pet.snap_visible(&displays);

    let mut window = PetWindow::create()?;
    let mut last_anim = Instant::now();
    let mut last_tick = Instant::now();
    let mut dragging = false;
    let mut last_pet_click = Instant::now()
        .checked_sub(std::time::Duration::from_secs(10))
        .unwrap_or_else(Instant::now);
    let mut pending_double = false;

    paint(&mut window, &mut player, &pet)?;

    window.run_blocking_until_quit(|window, event| {
        match event {
            WindowEvent::Quit => {}
            WindowEvent::RightUp => {
                let displays = list_displays();
                pet.snap_visible(&displays);
                let _ = paint(window, &mut player, &pet);
            }
            WindowEvent::LeftDown { x, y } => {
                let now = Instant::now();
                pending_double = now.duration_since(last_pet_click).as_millis() < 350;
                last_pet_click = now;
                dragging = true;
                pet.movement.set_clock_ms(now_ms());
                pet.begin_drag();
                pet.drag_to(Vec2::new(x as f32, y as f32));
                player.set_clip(pet.pose().anim);
                let _ = paint(window, &mut player, &pet);
            }
            WindowEvent::LeftUp { x, y } => {
                if dragging {
                    pet.movement.set_clock_ms(now_ms());
                    pet.drag_to(Vec2::new(x as f32, y as f32));
                    pet.end_drag();
                }
                dragging = false;
                player.set_clip(pet.pose().anim);
                let _ = paint(window, &mut player, &pet);
            }
            WindowEvent::MouseMove { x, y, dragging: d } => {
                if d || dragging {
                    pet.movement.set_clock_ms(now_ms());
                    pet.drag_to(Vec2::new(x as f32, y as f32));
                    player.set_clip(pet.pose().anim);
                    let _ = paint(window, &mut player, &pet);
                }
            }
            WindowEvent::Tick => {
                if dragging {
                    pet.movement.set_clock_ms(now_ms());
                    let dt = last_anim.elapsed().as_secs_f32().min(0.05);
                    last_anim = Instant::now();
                    player.update(dt);
                    let _ = paint(window, &mut player, &pet);
                    return;
                }
                let dt = last_tick.elapsed().as_secs_f32().min(0.05);
                last_tick = Instant::now();
                let displays = list_displays();
                let windows = list_window_surfaces();
                let window_count = windows.len();
                let double = pending_double;
                pending_double = false;
                let activity = perception.sample(dt, &displays, window_count, double);
                let pose = pet.tick(now_ms(), displays, windows, activity);
                let adt = last_anim.elapsed().as_secs_f32().min(0.05);
                last_anim = Instant::now();
                player.update(adt);
                player.set_clip(pose.anim);
                if let Err(err) = paint(window, &mut player, &pet) {
                    eprintln!("paint: {err}");
                }
            }
        }
    });

    Ok(())
}

fn paint(
    window: &mut PetWindow,
    player: &mut SpritePlayer,
    pet: &PetCore,
) -> Result<(), Box<dyn std::error::Error>> {
    let pose = pet.pose();
    let (bgra, w, h) = player.compose_bgra(&pose)?;
    window.present_bgra(&bgra, w, h, pose.x.round() as i32, pose.y.round() as i32)?;
    Ok(())
}

fn resolve_assets_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Ok(env) = std::env::var("DESKTOP_CAT_ASSETS") {
        return Ok(PathBuf::from(env));
    }
    let candidates = [
        PathBuf::from("assets/sprites/cat"),
        PathBuf::from("../../assets/sprites/cat"),
        std::env::current_exe()?
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("assets/sprites/cat"),
        std::env::current_dir()?.join("assets/sprites/cat"),
        std::env::current_dir()?.join("../assets/sprites/cat"),
        std::env::current_dir()?.join("../../assets/sprites/cat"),
    ];
    for c in candidates {
        if c.join("sheet.png").exists() {
            return Ok(c);
        }
    }
    Err("could not locate assets/sprites/cat (set DESKTOP_CAT_ASSETS)".into())
}
