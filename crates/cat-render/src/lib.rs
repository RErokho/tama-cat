//! Sprite sheet loading and BGRA frame composition for layered windows.

use cat_core::{AnimClip, PetPose, PET_FRAME_HEIGHT, PET_FRAME_WIDTH};
use image::RgbaImage;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RenderError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("image: {0}")]
    Image(#[from] image::ImageError),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("missing animation {0}")]
    MissingAnim(String),
}

#[derive(Debug, Deserialize)]
struct SheetMeta {
    #[serde(rename = "frameWidth")]
    frame_width: u32,
    #[serde(rename = "frameHeight")]
    frame_height: u32,
    animations: HashMap<String, AnimSpec>,
}

#[derive(Debug, Deserialize)]
struct AnimSpec {
    frames: Vec<u32>,
    fps: f32,
}

pub struct SpriteSheet {
    frames: Vec<RgbaImage>,
    animations: HashMap<String, AnimSpec>,
    frame_w: u32,
    frame_h: u32,
}

pub struct SpritePlayer {
    sheet: SpriteSheet,
    clip: AnimClip,
    frame_index: usize,
    accum: f32,
}

impl SpriteSheet {
    pub fn load(sheet_png: &Path, sheet_json: &Path) -> Result<Self, RenderError> {
        let meta: SheetMeta = serde_json::from_str(&std::fs::read_to_string(sheet_json)?)?;
        let img = image::open(sheet_png)?.to_rgba8();
        let fw = meta.frame_width;
        let fh = meta.frame_height;
        let cols = (img.width() / fw).max(1);
        let total = (img.width() / fw) * (img.height() / fh);
        let mut frames = Vec::with_capacity(total as usize);
        for i in 0..total {
            let col = i % cols;
            let row = i / cols;
            let frame = image::imageops::crop_imm(&img, col * fw, row * fh, fw, fh).to_image();
            frames.push(frame);
        }
        Ok(Self {
            frames,
            animations: meta.animations,
            frame_w: fw,
            frame_h: fh,
        })
    }
}

impl SpritePlayer {
    pub fn new(sheet: SpriteSheet) -> Self {
        Self {
            sheet,
            clip: AnimClip::Idle,
            frame_index: 0,
            accum: 0.0,
        }
    }

    pub fn set_clip(&mut self, clip: AnimClip) {
        if self.clip != clip {
            self.clip = clip;
            self.frame_index = 0;
            self.accum = 0.0;
        }
    }

    pub fn update(&mut self, dt: f32) {
        let key = self.clip.as_str();
        let Some(anim) = self.sheet.animations.get(key) else {
            return;
        };
        if anim.frames.is_empty() {
            return;
        }
        self.accum += dt;
        let frame_dur = 1.0 / anim.fps.max(1.0);
        while self.accum >= frame_dur {
            self.accum -= frame_dur;
            self.frame_index = (self.frame_index + 1) % anim.frames.len();
        }
    }

    /// Compose current pose into premultiplied BGRA buffer for UpdateLayeredWindow.
    pub fn compose_bgra(&mut self, pose: &PetPose) -> Result<(Vec<u8>, u32, u32), RenderError> {
        self.set_clip(pose.anim);
        let key = pose.anim.as_str();
        let anim = self
            .sheet
            .animations
            .get(key)
            .or_else(|| self.sheet.animations.get("idle"))
            .ok_or_else(|| RenderError::MissingAnim(key.to_string()))?;
        let idx = anim.frames[self.frame_index.min(anim.frames.len() - 1)] as usize;
        let src = self
            .sheet
            .frames
            .get(idx)
            .ok_or_else(|| RenderError::MissingAnim(format!("frame {idx}")))?;

        let out_w = ((PET_FRAME_WIDTH * pose.scale).round() as u32).max(1);
        let out_h = ((PET_FRAME_HEIGHT * pose.scale).round() as u32).max(1);
        let scaled = if out_w == self.sheet.frame_w && out_h == self.sheet.frame_h {
            src.clone()
        } else {
            image::imageops::resize(src, out_w, out_h, image::imageops::FilterType::Triangle)
        };

        let mut flipped = scaled;
        if pose.facing < 0 {
            flipped = image::imageops::flip_horizontal(&flipped);
        }

        let mut bgra = Vec::with_capacity((out_w * out_h * 4) as usize);
        for px in flipped.pixels() {
            let [r, g, b, a] = px.0;
            // Premultiply for layered window
            let pr = (r as u16 * a as u16 / 255) as u8;
            let pg = (g as u16 * a as u16 / 255) as u8;
            let pb = (b as u16 * a as u16 / 255) as u8;
            bgra.extend_from_slice(&[pb, pg, pr, a]);
        }
        Ok((bgra, out_w, out_h))
    }
}
