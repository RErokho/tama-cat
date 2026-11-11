use crate::config::SURFACE_POLL_HZ;
use crate::types::{DisplayInfo, Surface, SurfaceKind};

/// Collects walkable surfaces. Platform injects window tops via callback.
pub struct WorldSystem {
    surfaces: Vec<Surface>,
    displays: Vec<DisplayInfo>,
    poll_accum: f32,
    poll_interval: f32,
}

impl Default for WorldSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl WorldSystem {
    pub fn new() -> Self {
        Self {
            surfaces: Vec::new(),
            displays: Vec::new(),
            poll_accum: 0.0,
            poll_interval: 1.0 / SURFACE_POLL_HZ,
        }
    }

    pub fn surfaces(&self) -> &[Surface] {
        &self.surfaces
    }

    pub fn displays(&self) -> &[DisplayInfo] {
        &self.displays
    }

    pub fn refresh(
        &mut self,
        displays: Vec<DisplayInfo>,
        window_surfaces: Vec<Surface>,
    ) {
        self.displays = displays;
        let mut surfaces = Vec::new();
        for d in &self.displays {
            surfaces.push(Surface {
                id: format!("floor-{}", d.id),
                kind: SurfaceKind::Floor,
                x: d.left,
                y: d.floor_y,
                width: d.right - d.left,
                height: 2.0,
            });
            let bottom_bar = d.bottom - d.floor_y;
            if bottom_bar > 8.0 {
                surfaces.push(Surface {
                    id: format!("taskbar-{}", d.id),
                    kind: SurfaceKind::Taskbar,
                    x: d.left,
                    y: d.bottom - bottom_bar.max(40.0),
                    width: d.right - d.left,
                    height: bottom_bar.max(40.0),
                });
            }
            surfaces.push(Surface {
                id: format!("edge-top-{}", d.id),
                kind: SurfaceKind::ScreenEdge,
                x: d.left,
                y: d.top + 2.0,
                width: d.right - d.left,
                height: 2.0,
            });
        }
        surfaces.extend(window_surfaces);
        self.surfaces = surfaces;
    }

    pub fn update(
        &mut self,
        dt: f32,
        displays: Vec<DisplayInfo>,
        window_surfaces: Vec<Surface>,
    ) {
        self.poll_accum += dt;
        if self.poll_accum >= self.poll_interval {
            self.poll_accum = 0.0;
            self.refresh(displays, window_surfaces);
        }
    }
}
