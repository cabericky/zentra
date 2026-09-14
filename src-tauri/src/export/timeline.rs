use crate::storage::{ClickRecord, PanSampleRecord};

/// Cubic ease-in-out curve for smooth acceleration and deceleration.
#[inline]
pub fn cubic_ease_in_out(mut t: f64) -> f64 {
    t = t.clamp(0.0, 1.0);
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

/// A dynamic zoom & pan state at a specific point in time.
#[derive(Debug, Clone, Copy)]
pub struct TransformState {
    pub scale: f32,
    pub center_x: f32,
    pub center_y: f32,
}

#[derive(Debug, Clone)]
struct KeyframeSegment {
    start_time: f64,
    end_time: f64,
    from_scale: f64,
    to_scale: f64,
    from_x: f64,
    to_x: f64,
    from_y: f64,
    to_y: f64,
}

pub struct KeyframeTimeline {
    screen_width: f64,
    screen_height: f64,
    segments: Vec<KeyframeSegment>,
    pan_samples: Vec<PanSampleRecord>,
}

impl KeyframeTimeline {
    /// Builds a keyframe timeline from recorded clicks.
    pub fn build(clicks: &[ClickRecord], screen_width: u32, screen_height: u32) -> Self {
        Self::build_with_pan_samples(clicks, &[], screen_width, screen_height)
    }

    /// Builds a keyframe timeline incorporating both recorded clicks and continuous pan/drag samples.
    pub fn build_with_pan_samples(
        clicks: &[ClickRecord],
        pan_samples: &[PanSampleRecord],
        screen_width: u32,
        screen_height: u32,
    ) -> Self {
        let sw = screen_width as f64;
        let sh = screen_height as f64;
        let center_x = sw / 2.0;
        let center_y = sh / 2.0;

        let ease_duration = 0.35;
        let peak_scale = 1.8;

        let mut segments = Vec::new();
        let mut cur_time = 0.0;
        let mut cur_scale = 1.0;
        let mut cur_x = center_x;
        let mut cur_y = center_y;

        for click in clicks {
            let t = click.time_seconds;

            if click.button == "zoom_in" || click.button == "backtick" {
                let target_x = click.x as f64;
                let target_y = click.y as f64;

                if cur_scale < 1.05 {
                    // Zoom-in transition from 1.0 to peak_scale
                    let trans_start = (t - 0.20).max(cur_time);
                    if trans_start > cur_time {
                        segments.push(KeyframeSegment {
                            start_time: cur_time,
                            end_time: trans_start,
                            from_scale: 1.0,
                            to_scale: 1.0,
                            from_x: center_x,
                            to_x: center_x,
                            from_y: center_y,
                            to_y: center_y,
                        });
                    }
                    let trans_end = trans_start + ease_duration;
                    segments.push(KeyframeSegment {
                        start_time: trans_start,
                        end_time: trans_end,
                        from_scale: 1.0,
                        to_scale: peak_scale,
                        from_x: center_x,
                        to_x: target_x,
                        from_y: center_y,
                        to_y: target_y,
                    });
                    cur_time = trans_end;
                    cur_scale = peak_scale;
                    cur_x = target_x;
                    cur_y = target_y;
                } else {
                    // Already zoomed in -> pan transition at peak_scale
                    let trans_start = (t - 0.20).max(cur_time);
                    if trans_start > cur_time {
                        segments.push(KeyframeSegment {
                            start_time: cur_time,
                            end_time: trans_start,
                            from_scale: peak_scale,
                            to_scale: peak_scale,
                            from_x: cur_x,
                            to_x: cur_x,
                            from_y: cur_y,
                            to_y: cur_y,
                        });
                    }
                    let trans_end = trans_start + ease_duration;
                    segments.push(KeyframeSegment {
                        start_time: trans_start,
                        end_time: trans_end,
                        from_scale: peak_scale,
                        to_scale: peak_scale,
                        from_x: cur_x,
                        to_x: target_x,
                        from_y: cur_y,
                        to_y: target_y,
                    });
                    cur_time = trans_end;
                    cur_scale = peak_scale;
                    cur_x = target_x;
                    cur_y = target_y;
                }
            } else if click.button == "zoom_out" || click.button == "escape" {
                if cur_scale > 1.05 {
                    // Zoom-out transition from peak_scale back to 1.0
                    let trans_start = t.max(cur_time);
                    if trans_start > cur_time {
                        segments.push(KeyframeSegment {
                            start_time: cur_time,
                            end_time: trans_start,
                            from_scale: peak_scale,
                            to_scale: peak_scale,
                            from_x: cur_x,
                            to_x: cur_x,
                            from_y: cur_y,
                            to_y: cur_y,
                        });
                    }
                    let trans_end = trans_start + ease_duration;
                    segments.push(KeyframeSegment {
                        start_time: trans_start,
                        end_time: trans_end,
                        from_scale: peak_scale,
                        to_scale: 1.0,
                        from_x: cur_x,
                        to_x: center_x,
                        from_y: cur_y,
                        to_y: center_y,
                    });
                    cur_time = trans_end;
                    cur_scale = 1.0;
                    cur_x = center_x;
                    cur_y = center_y;
                }
            }
        }

        // Natural fallback zoom-out if user never pressed escape
        if cur_scale > 1.05 {
            let hold_end = cur_time + 2.0;
            segments.push(KeyframeSegment {
                start_time: cur_time,
                end_time: hold_end,
                from_scale: peak_scale,
                to_scale: peak_scale,
                from_x: cur_x,
                to_x: cur_x,
                from_y: cur_y,
                to_y: cur_y,
            });
            let trans_end = hold_end + ease_duration;
            segments.push(KeyframeSegment {
                start_time: hold_end,
                end_time: trans_end,
                from_scale: peak_scale,
                to_scale: 1.0,
                from_x: cur_x,
                to_x: center_x,
                from_y: cur_y,
                to_y: center_y,
            });
        }

        Self {
            screen_width: sw,
            screen_height: sh,
            segments,
            pan_samples: pan_samples.to_vec(),
        }
    }

    /// Evaluates the zoom scale and pan center at time `t_seconds`.
    pub fn evaluate(&self, t: f64) -> TransformState {
        let center_x = self.screen_width / 2.0;
        let center_y = self.screen_height / 2.0;

        let (raw_scale, mut raw_cx, mut raw_cy) = if !self.pan_samples.is_empty() {
            // High-fidelity camera pan path recorded during the session
            if t <= self.pan_samples[0].time_seconds {
                let first = &self.pan_samples[0];
                if first.scale > 1.001 && (first.time_seconds - t).abs() <= 0.35 {
                    let progress = ((0.35 - (first.time_seconds - t)) / 0.35).clamp(0.0, 1.0);
                    let eased = cubic_ease_in_out(progress);
                    let s = 1.0 + (first.scale as f64 - 1.0) * eased;
                    let cx = center_x + (first.center_x as f64 - center_x) * eased;
                    let cy = center_y + (first.center_y as f64 - center_y) * eased;
                    (s, cx, cy)
                } else {
                    (1.0, center_x, center_y)
                }
            } else if t >= self.pan_samples.last().unwrap().time_seconds {
                let last = self.pan_samples.last().unwrap();
                if last.scale > 1.001 && (t - last.time_seconds) <= 0.35 {
                    let progress = ((t - last.time_seconds) / 0.35).clamp(0.0, 1.0);
                    let eased = cubic_ease_in_out(1.0 - progress);
                    let s = 1.0 + (last.scale as f64 - 1.0) * eased;
                    let cx = center_x + (last.center_x as f64 - center_x) * eased;
                    let cy = center_y + (last.center_y as f64 - center_y) * eased;
                    (s, cx, cy)
                } else {
                    (1.0, center_x, center_y)
                }
            } else {
                let idx = match self.pan_samples.binary_search_by(|s| {
                    s.time_seconds.partial_cmp(&t).unwrap_or(std::cmp::Ordering::Equal)
                }) {
                    Ok(i) => i,
                    Err(i) => if i > 0 { i - 1 } else { 0 },
                };

                let s1 = &self.pan_samples[idx];
                let next_idx = (idx + 1).min(self.pan_samples.len() - 1);
                let s2 = &self.pan_samples[next_idx];
                let dt = s2.time_seconds - s1.time_seconds;

                if s1.scale <= 1.001 && s2.scale <= 1.001 {
                    (1.0, center_x, center_y)
                } else if dt > 0.40 {
                    if t - s1.time_seconds <= 0.35 && s1.scale > 1.001 {
                        let progress = ((t - s1.time_seconds) / 0.35).clamp(0.0, 1.0);
                        let eased = cubic_ease_in_out(1.0 - progress);
                        let s = 1.0 + (s1.scale as f64 - 1.0) * eased;
                        let cx = center_x + (s1.center_x as f64 - center_x) * eased;
                        let cy = center_y + (s1.center_y as f64 - center_y) * eased;
                        (s, cx, cy)
                    } else if s2.time_seconds - t <= 0.35 && s2.scale > 1.001 {
                        let progress = ((0.35 - (s2.time_seconds - t)) / 0.35).clamp(0.0, 1.0);
                        let eased = cubic_ease_in_out(progress);
                        let s = 1.0 + (s2.scale as f64 - 1.0) * eased;
                        let cx = center_x + (s2.center_x as f64 - center_x) * eased;
                        let cy = center_y + (s2.center_y as f64 - center_y) * eased;
                        (s, cx, cy)
                    } else {
                        (1.0, center_x, center_y)
                    }
                } else {
                    let alpha = if dt > 1e-6 {
                        ((t - s1.time_seconds) / dt).clamp(0.0, 1.0)
                    } else {
                        0.0
                    };
                    let s = s1.scale as f64 + (s2.scale as f64 - s1.scale as f64) * alpha;
                    let cx = s1.center_x as f64 + (s2.center_x as f64 - s1.center_x as f64) * alpha;
                    let cy = s1.center_y as f64 + (s2.center_y as f64 - s1.center_y as f64) * alpha;
                    (s, cx, cy)
                }
            }
        } else if self.segments.is_empty() {
            (1.0, center_x, center_y)
        } else if t <= self.segments[0].start_time {
            (1.0, center_x, center_y)
        } else if t >= self.segments.last().unwrap().end_time {
            (1.0, center_x, center_y)
        } else {
            // Find active segment
            let seg = self.segments.iter().find(|s| t >= s.start_time && t <= s.end_time);
            if let Some(s) = seg {
                let dur = s.end_time - s.start_time;
                let progress = if dur > 1e-6 {
                    (t - s.start_time) / dur
                } else {
                    1.0
                };
                let eased = cubic_ease_in_out(progress);
                let scale = s.from_scale + (s.to_scale - s.from_scale) * eased;
                let cx = s.from_x + (s.to_x - s.from_x) * eased;
                let cy = s.from_y + (s.to_y - s.from_y) * eased;
                (scale, cx, cy)
            } else {
                (1.0, center_x, center_y)
            }
        };

        // Strict boundary clamping so camera never shows black margins
        let half_w = self.screen_width / (2.0 * raw_scale);
        let half_h = self.screen_height / (2.0 * raw_scale);
        raw_cx = raw_cx.clamp(half_w, self.screen_width - half_w);
        raw_cy = raw_cy.clamp(half_h, self.screen_height - half_h);

        TransformState {
            scale: raw_scale as f32,
            center_x: raw_cx as f32,
            center_y: raw_cy as f32,
        }
    }
}