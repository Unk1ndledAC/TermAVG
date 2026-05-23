use std::time::Duration;

use tmj_core::script::TypeName;

use crate::pages::behaviour::{
    animation::{Animation, AnyAnimation, anim_normalized_time, rect_trans::RectTransCurve},
    visual_element::VisualElement,
};

fn lerp_i32(a: i32, b: i32, t: f32) -> i32 {
    let af = f64::from(a);
    let bf = f64::from(b);
    (af + (bf - af) * f64::from(t)).round() as i32
}

fn lerp_offset(a: (i32, i32), b: (i32, i32), t: f32) -> (i32, i32) {
    (lerp_i32(a.0, b.0, t), lerp_i32(a.1, b.1, t))
}

#[derive(strum::EnumString, strum::Display,)]
pub enum ShiftDirection {
    Up,
    Down,
    Left,
    Right
}

impl ShiftDirection {
    pub fn apply(&self, begin: (i32, i32), distance: i64) -> (i32, i32) {
        let distance = distance as i32;
        match self {
            ShiftDirection::Up => {
                (begin.0, begin.1 - distance)
            }
            ShiftDirection::Down => {

                (begin.0, begin.1 + distance)
            }
            ShiftDirection::Left => {
                (begin.0 - distance, begin.1)
            }
            ShiftDirection::Right => {
                (begin.0 + distance, begin.1)
            },
        }
    }
}

/// 在 `anim_time` 内将 `VisualElement.offset` 从 `start_offset` 插值到 `target_offset`。
#[derive(TypeName, Default)]
pub struct OffsetShift {
    pub anim_time: Duration,
    pub start_offset: (i32, i32),
    pub target_offset: (i32, i32),
    pub curve: RectTransCurve,
    pub run_time: Duration,
}

impl OffsetShift {
    pub fn begin(
        &mut self,
        start: (i32, i32),
        target: (i32, i32),
        duration_secs: f64,
        curve: RectTransCurve,
    ) {
        self.start_offset = start;
        self.target_offset = target;
        self.curve = curve;
        self.anim_time = Duration::from_secs_f64(duration_secs.max(0.0));
        self.run_time = Duration::ZERO;
    }

    pub fn new(
        start: (i32, i32),
        target: (i32, i32),
        duration: Duration,
        curve: RectTransCurve,
    ) -> Self {
        let mut s = Self::default();
        s.begin(start, target, duration.as_secs_f64(), curve);
        s
    }
}

impl AnyAnimation for OffsetShift {}

impl Animation for OffsetShift {
    fn apply_to_ve(&self, ve: &mut VisualElement) -> anyhow::Result<()> {
        let eased = self
            .curve
            .apply(anim_normalized_time(self.run_time, self.anim_time));
        ve.offset = lerp_offset(self.start_offset, self.target_offset, eased);
        Ok(())
    }

    fn update(&mut self, tick_delta: Duration) {
        self.run_time += tick_delta;
        self.run_time = self.run_time.min(self.anim_time);
    }

    fn force_over(&mut self) {
        self.run_time = self.anim_time;
    }

    fn reset(&mut self) {
        self.run_time = Duration::ZERO;
    }

    fn is_animing(&self) -> bool {
        self.run_time < self.anim_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lerp_offset_endpoints() {
        assert_eq!(lerp_offset((0, 0), (10, -4), 0.0), (0, 0));
        assert_eq!(lerp_offset((0, 0), (10, -4), 1.0), (10, -4));
    }
}
