use std::time::{self, Duration};

use tmj_core::script::TypeName;

use crate::pages::behaviour::animation::{Animation, AnyAnimation};

#[derive(TypeName, Default)]
pub struct EffectBytesStream{
    pub run_time: time::Duration,
}

impl EffectBytesStream{}

impl Animation for EffectBytesStream{
    fn apply_to_ve(
        &self,
        ve: &mut crate::pages::behaviour::visual_element::VisualElement,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    fn update(&mut self, tick_delta: std::time::Duration) {
        self.run_time += tick_delta;
    }

    fn force_over(&mut self) {
    }

    fn reset(&mut self) {
    }

    fn is_animing(&self) -> bool {
        false
    }
    fn is_indeterminate(&self) -> bool {
        true
    }
}

impl AnyAnimation for EffectBytesStream{
}
