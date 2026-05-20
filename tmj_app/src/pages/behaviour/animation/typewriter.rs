use std::time::{self, Duration};

use tmj_core::script::TypeName;

use crate::pages::behaviour::{animation::Animation, visual_element::VisualElementKind};

#[derive(TypeName, Default)]
pub struct AniTypeWriter {
    pub anim_time: f64,
    pub start_text: String,
    pub target_text: String,
    pub speed: f64,
    pub run_time: time::Duration,
}

impl AniTypeWriter {
    fn diff_chars(&self) -> usize {
        self.target_text
            .chars()
            .count()
            .saturating_sub(self.start_text.chars().count())
    }

    fn target_char_count(&self) -> usize {
        self.target_text.chars().count()
    }

    fn start_char_count(&self) -> usize {
        self.start_text.chars().count()
    }

    /// 与 `apply_to_ve` 使用同一套公式，避免「字已打完但仍 is_animing」。
    fn displayed_char_count(&self) -> usize {
        let target_total = self.target_char_count();
        if self.speed <= 0.0 {
            return target_total;
        }
        let elapsed_secs = self.run_time.as_secs_f64().max(0.0);
        let shown_chars = self.start_char_count() as f64 + elapsed_secs * self.speed;
        shown_chars
            .ceil()
            .clamp(0.0, target_total as f64) as usize
    }

    fn anim_time(&self) -> Duration {
        if self.speed <= 0.0 {
            return Duration::ZERO;
        }
        let diff = self.diff_chars();
        if diff == 0 {
            return Duration::ZERO;
        }
        Duration::from_secs_f64(diff as f64 / self.speed)
    }
}

impl Animation for AniTypeWriter {
    fn apply_to_ve(
        &self,
        ve: &mut crate::pages::behaviour::visual_element::VisualElement,
    ) -> anyhow::Result<()> {
        if let VisualElementKind::Text { content } = &mut ve.kind {
            let shown_chars = self.displayed_char_count();
            *content = self
                .target_text
                .chars()
                .take(shown_chars)
                .collect::<String>();
        }
        Ok(())
    }

    fn update(&mut self, tick_delta: std::time::Duration) {
        self.run_time += tick_delta;
        self.run_time = self.run_time.clamp(Duration::ZERO, self.anim_time());
    }

    fn force_over(&mut self) {
        self.run_time = self.anim_time();
    }

    fn reset(&mut self) {
        self.run_time = Duration::ZERO;
        self.start_text = "".into();
        self.target_text = "".into();
    }

    fn is_animing(&self) -> bool {
        if self.target_text == self.start_text {
            return false;
        }
        self.displayed_char_count() < self.target_char_count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> AniTypeWriter {
        AniTypeWriter {
            start_text: "".into(),
            target_text: "abcd".into(),
            speed: 40.0,
            run_time: Duration::ZERO,
            ..Default::default()
        }
    }

    #[test]
    fn displayed_chars_match_apply_before_anim_time() {
        let tw = sample();
        let mid = Duration::from_secs_f64(tw.anim_time().as_secs_f64() * 0.5);
        let mut tw_mid = sample();
        tw_mid.run_time = mid;
        assert!(tw_mid.displayed_char_count() < tw.target_char_count());
    }

    #[test]
    fn full_text_not_animating_at_anim_time() {
        let mut tw = sample();
        tw.run_time = tw.anim_time();
        assert_eq!(tw.displayed_char_count(), tw.target_char_count());
        assert!(!tw.is_animing());
    }

    #[test]
    fn force_over_finishes_immediately() {
        let mut tw = sample();
        tw.force_over();
        assert!(!tw.is_animing());
        assert_eq!(tw.displayed_char_count(), tw.target_char_count());
    }
}
