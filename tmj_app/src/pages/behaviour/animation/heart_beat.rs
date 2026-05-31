use std::time::{self, Duration};

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::Color,
};
use tmj_core::script::TypeName;

use crate::{
    art::theme::THEME,
    pages::behaviour::{
        animation::{Animation, AnyAnimation},
        visual_element::{VisualElement, VisualElementCustomDrawer, VisualElementKind},
    },
};

/// 脉冲间隔（秒）
const PULSE_INTERVAL: f64 = 3.0;
/// 环宽（格）
const RING_WIDTH: f64 = 5.0;
/// 终端字符宽高比（宽:高）
const CHAR_ASPECT: f64 = 0.5;
/// 环字符，从内到外
const RING_CHARS: &[u8] = b"..ooOO00OOoo..";
/// 半径渐近目标（格）
const RADIUS_TARGET: f64 = 40.0;
/// 扩张速率：值越大起爆越猛
const RATE: f64 = 1.5;

/// 脉冲半径：快速起爆后逐渐减速（指数饱和）
fn pulse_radius(t: f64) -> f64 {
    RADIUS_TARGET * (1.0 - (-t * RATE).exp())
}

/// 脉冲亮度：起爆闪亮后随半径平滑衰减
fn pulse_brightness(t: f64) -> f64 {
    let t = t.max(0.0);
    let flash = (-(t / 0.12).powi(2)).exp();
    let r = pulse_radius(t);
    let decay = (1.0 - r / RADIUS_TARGET).max(0.0);
    (flash * 0.3 + 0.7 * decay).min(1.0)
}

/// 心跳波纹：每 2 秒一个字符脉冲椭圆环从中心扩散
#[derive(TypeName, Default)]
pub struct EffectHeartBeat {
    pub run_time: time::Duration,
}

impl EffectHeartBeat {
    fn ensure_custom_drawer(ve: &mut VisualElement) {
        if !matches!(ve.kind, VisualElementKind::Custom { .. }) {
            ve.kind = VisualElementKind::Custom {
                drawer: VisualElementCustomDrawer::from(|_, _, _| Ok(())),
            };
        }
        ve.clear_before_draw = false;
    }

    fn draw(
        _ve: &VisualElement,
        buffer: &mut Buffer,
        rect: Rect,
        t: f64,
    ) -> anyhow::Result<()> {
        if rect.width == 0 || rect.height == 0 {
            return Ok(());
        }

        let cx = rect.x as f64 + rect.width as f64 / 2.0;
        let cy = rect.y as f64 + rect.height as f64 / 2.0;
        let ring_chars_len = RING_CHARS.len();

        let pulse_count = (t / PULSE_INTERVAL).floor() as u32;
        let start = if pulse_count > 3 { pulse_count - 3 } else { 0 };

        for i in start..=pulse_count {
            let age = t - i as f64 * PULSE_INTERVAL;
            if age < 0.0 {
                continue;
            }

            let r = pulse_radius(age);
            if r <= 0.0 || r > RADIUS_TARGET {
                continue;
            }

            let inner = (r - RING_WIDTH / 2.0).max(0.0);
            let outer = r + RING_WIDTH / 2.0;
            let ring_w = outer - inner;
            let bri = pulse_brightness(age);

            let y0 = (cy - outer - 1.0).floor() as u16;
            let y1 = (cy + outer + 1.0).ceil() as u16;
            let x0 = (cx - outer / CHAR_ASPECT - 1.0).floor() as u16;
            let x1 = (cx + outer / CHAR_ASPECT + 1.0).ceil() as u16;

            for y in y0.max(rect.y)..y1.min(rect.bottom()) {
                for x in x0.max(rect.x)..x1.min(rect.right()) {
                    let dx = (x as f64 - cx) * CHAR_ASPECT;
                    let dy = y as f64 - cy;
                    let d = (dx * dx + dy * dy).sqrt();

                    if d >= inner && d <= outer {
                        let offset = ((d - inner) / ring_w * (ring_chars_len - 1) as f64) as usize;
                        let ch = RING_CHARS[offset.min(ring_chars_len - 1)] as char;

                        let cell = &mut buffer[(x, y)];
                        cell.set_symbol(ch.encode_utf8(&mut [0u8; 4]));
                        if let Color::Rgb(r, g, b) = THEME.dialouge.heart_beat.fg.unwrap_or(Color::Rgb(220, 27, 27)) {
                            let scale = bri;
                            cell.set_fg(Color::Rgb(
                                (r as f64 * scale) as u8,
                                (g as f64 * scale) as u8,
                                (b as f64 * scale) as u8,
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl Animation for EffectHeartBeat {
    fn apply_to_ve(
        &self,
        ve: &mut VisualElement,
    ) -> anyhow::Result<()> {
        Self::ensure_custom_drawer(ve);
        let t = self.run_time.as_secs_f64();
        if let VisualElementKind::Custom { drawer } = &mut ve.kind {
            drawer.draw = Box::new(move |ve, buffer, rect| Self::draw(ve, buffer, rect, t));
        }
        Ok(())
    }

    fn update(&mut self, tick_delta: Duration) {
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

impl AnyAnimation for EffectHeartBeat {}
