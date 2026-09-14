#[derive(Debug, Clone, Copy)]
pub enum MagnifierCmd {
    ZoomIn { x: i32, y: i32 },
    ZoomOut,
    Stop,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub mag: f32,
    pub offset_x: f32,
    pub offset_y: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            mag: 1.0,
            offset_x: 0.0,
            offset_y: 0.0,
        }
    }
}

/// Evaluates Hermite cubic smoothstep easing: 3t² - 2t³
pub fn hermite_smoothstep(t: f32) -> f32 {
    let clamped = t.clamp(0.0, 1.0);
    clamped * clamped * (3.0 - 2.0 * clamped)
}

/// Interpolates between `from` and `target` transforms using Hermite smoothstep easing.
pub fn interpolate_transform(from: Transform, target: Transform, progress: f32) -> Transform {
    let eased = hermite_smoothstep(progress);
    Transform {
        mag: from.mag + (target.mag - from.mag) * eased,
        offset_x: from.offset_x + (target.offset_x - from.offset_x) * eased,
        offset_y: from.offset_y + (target.offset_y - from.offset_y) * eased,
    }
}

/// Clamps target camera coordinates within display boundaries at the specified magnification factor.
pub fn calculate_target_offset(center_x: f32, center_y: f32, screen_w: f32, screen_h: f32, mag: f32) -> (f32, f32) {
    let vis_w = screen_w / mag;
    let vis_h = screen_h / mag;
    let target_x = (center_x - vis_w * 0.5).clamp(0.0, screen_w - vis_w);
    let target_y = (center_y - vis_h * 0.5).clamp(0.0, screen_h - vis_h);
    (target_x, target_y)
}