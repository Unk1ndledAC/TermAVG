use std::time::{self, Duration};

use tmj_core::script::TypeName;

use crate::pages::behaviour::animation::{Animation, AnyAnimation};

#[derive(TypeName, Default)]
pub struct AniAlpha {
    pub anim_time: time::Duration,
    pub start_alpha: f64,
    pub target_alpha: f64,
    pub run_time: time::Duration,
}

impl AniAlpha {
    pub fn new(start: f64, end: f64, time: time::Duration) -> Self{
        AniAlpha {
            anim_time: time,
            start_alpha: start,
            target_alpha: end,
            ..Default::default()
        }
    }
}
impl AnyAnimation for AniAlpha {}
impl Animation for AniAlpha {
    fn apply_to_ve(
        &self,
        ve: &mut crate::pages::behaviour::visual_element::VisualElement,
    ) -> anyhow::Result<()> {
        let elapsed_secs = self.run_time.as_secs_f64().max(0.0);
        let total_secs = self.anim_time.as_secs_f64().max(0.0);
        let mut evalued_alpha = if total_secs <= 0.0 {
            self.target_alpha
        } else {
            self.start_alpha
                + (self.target_alpha - self.start_alpha) * (elapsed_secs / total_secs)
        };
        let alpha_max = self.start_alpha.max(self.target_alpha);
        evalued_alpha = evalued_alpha.clamp(0.0, alpha_max);
        ve.alpha = evalued_alpha;
        if evalued_alpha <= 0.001 {
            ve.visible = false
        } else {
            ve.visible = true
        }
        Ok(())
    }

    fn update(&mut self, tick_delta: std::time::Duration) {
        if self.anim_time.is_zero() {
            self.run_time = Duration::ZERO;
            self.start_alpha = self.target_alpha;
            return;
        }
        self.run_time += tick_delta;
        self.run_time = self.run_time.clamp(Duration::ZERO, self.anim_time);
        if self.run_time >= self.anim_time {
            self.start_alpha = self.target_alpha;
        }
    }

    fn force_over(&mut self) {
        self.run_time = self.anim_time;
        self.start_alpha = self.target_alpha;
    }

    fn reset(&mut self) {
        self.run_time = Duration::ZERO;
        self.anim_time = Duration::ZERO;
        self.start_alpha = self.target_alpha;
    }

    fn is_animing(&self) -> bool {
        !self.anim_time.is_zero() && self.run_time < self.anim_time
    }
}
