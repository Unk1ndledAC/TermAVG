use std::time::{self, Duration};

use rand::{Rng, SeedableRng};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
};
use tmj_core::script::TypeName;

use crate::pages::behaviour::{
    animation::{Animation, AnyAnimation},
    visual_element::{VisualElement, VisualElementCustomDrawer, VisualElementKind},
};

/// 每帧最多扰乱多少个像素（实际数量还会根据帧数 ramp up）
const GLITCH_SPEED: usize = 300;
/// 扰乱半径（像素偏移的最大曼哈顿距离）
const GLITCH_RADIUS: u16 = 6;
/// 色彩偏移的概率（其余概率执行像素偏移）
const COLOR_SHIFT_RATIO: f64 = 0.4;

/// 扰乱特效：随机偏移 buffer 像素并偏移像素色彩
#[derive(TypeName, Default)]
pub struct EffectGlitch {
    pub run_time: time::Duration,
}

impl EffectGlitch {
    fn ensure_custom_drawer(ve: &mut VisualElement) {
        if !matches!(ve.kind, VisualElementKind::Custom { .. }) {
            ve.kind = VisualElementKind::Custom {
                drawer: VisualElementCustomDrawer::from(|_, _, _| Ok(())),
            };
        }
        ve.clear_before_draw = false;
    }

    fn draw_glitch(
        _ve: &VisualElement,
        buffer: &mut Buffer,
        rect: Rect,
        t: f64,
    ) -> anyhow::Result<()> {
        if rect.width < 2 || rect.height < 2 {
            return Ok(());
        }

        let area = rect.width.saturating_sub(1) * rect.height.saturating_sub(1);
        if area == 0 {
            return Ok(());
        }

        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64((t * 100.0) as u64);

        // 强度随时间正弦波动，0.3 ~ 1.0
        let intensity = 0.3 + 0.7 * ((t * 0.8).sin() * 0.5 + 0.5);

        let pixel_count =
            ((GLITCH_SPEED as f64 * intensity) as usize).min(area as usize / 2);

        for _ in 0..pixel_count {
            let x = rng.random_range(rect.left()..rect.right());
            let y = rng.random_range(rect.top()..rect.bottom());

            if rng.random_bool(COLOR_SHIFT_RATIO) {
                let cell = &mut buffer[(x, y)];
                let mut style = cell.style();
                if let Some(Color::Rgb(r, g, b)) = style.fg {
                    let nr = (r as i16 + rng.random_range(-40..=40)).clamp(0, 255) as u8;
                    let ng = (g as i16 + rng.random_range(-40..=40)).clamp(0, 255) as u8;
                    let nb = (b as i16 + rng.random_range(-40..=40)).clamp(0, 255) as u8;
                    style.fg = Some(Color::Rgb(nr, ng, nb));
                }
                if let Some(Color::Rgb(r, g, b)) = style.bg {
                    let nr = (r as i16 + rng.random_range(-40..=40)).clamp(0, 255) as u8;
                    let ng = (g as i16 + rng.random_range(-40..=40)).clamp(0, 255) as u8;
                    let nb = (b as i16 + rng.random_range(-40..=40)).clamp(0, 255) as u8;
                    style.bg = Some(Color::Rgb(nr, ng, nb));
                }
                cell.set_style(style);
            } else {
                let dx = rng.random_range(-(GLITCH_RADIUS as i16)..=GLITCH_RADIUS as i16);
                let dy = rng.random_range(-(GLITCH_RADIUS as i16)..=GLITCH_RADIUS as i16);

                let dest_x = (x as i16 + dx).clamp(rect.left() as i16, rect.right() as i16 - 1) as u16;
                let dest_y = (y as i16 + dy).clamp(rect.top() as i16, rect.bottom() as i16 - 1) as u16;

                if (dest_x, dest_y) == (x, y) {
                    continue;
                }

                let src = buffer[(x, y)].clone();
                buffer[(dest_x, dest_y)] = src;
            }
        }

        Ok(())
    }
}

impl Animation for EffectGlitch {
    fn apply_to_ve(
        &self,
        ve: &mut VisualElement,
    ) -> anyhow::Result<()> {
        Self::ensure_custom_drawer(ve);
        let t = self.run_time.as_secs_f64();
        if let VisualElementKind::Custom { drawer } = &mut ve.kind {
            drawer.draw = Box::new(move |ve, buffer, rect| Self::draw_glitch(ve, buffer, rect, t));
        }
        Ok(())
    }

    fn update(&mut self, tick_delta: std::time::Duration) {
        self.run_time += tick_delta;
    }

    fn force_over(&mut self) {}

    fn reset(&mut self) {
        self.run_time = Duration::ZERO;
    }

    fn is_animing(&self) -> bool {
        false
    }

    fn is_indeterminate(&self) -> bool {
        true
    }
}

impl AnyAnimation for EffectGlitch {}
