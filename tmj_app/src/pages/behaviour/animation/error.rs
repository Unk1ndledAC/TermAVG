use std::time::{self, Duration};

use ratatui::{
    buffer::{Buffer, Cell},
    layout::Rect,
    style::Color,
};
use tmj_core::script::TypeName;

use crate::{
    art::theme,
    pages::behaviour::{
        animation::{Animation, AnyAnimation},
        visual_element::{VisualElement, VisualElementCustomDrawer, VisualElementKind},
    },
};

const BANNER: &str = "!!! CRITICAL SYSTEM ERROR !!!  >>> MALFUNCTION DETECTED <<<  ";
const CENTER_BAND: &str = "█▓▒░ ERROR WARNING ░▒▓█";

#[derive(TypeName, Default)]
pub struct EffectError {
    pub run_time: time::Duration,
}

impl EffectError {
    fn ensure_custom_drawer(ve: &mut VisualElement) {
        if !matches!(ve.kind, VisualElementKind::Custom { .. }) {
            ve.kind = VisualElementKind::Custom {
                drawer: VisualElementCustomDrawer::from(|_, _, _| Ok(())),
            };
        }
        ve.clear_before_draw = true;
    }

    fn draw_warning(
        _ve: &VisualElement,
        buffer: &mut Buffer,
        rect: Rect,
        t: f64,
    ) -> anyhow::Result<()> {
        if rect.width == 0 || rect.height == 0 {
            return Ok(());
        }

        let flash_on = (t * 5.5).sin() > 0.0;
        let bright_red = Color::Rgb(255, 40, 40);
        let dim_red = Color::Rgb(120, 16, 16);
        let fg = if flash_on { bright_red } else { dim_red };
        let bg = if flash_on {
            Color::Rgb(28, 0, 0)
        } else {
            theme::BLACK
        };
        let accent_bg = Color::Rgb(56, 0, 0);

        let mut fill = Cell::new(" ");
        fill.set_fg(fg);
        fill.set_bg(bg);
        for row in rect.rows() {
            for col in row.columns() {
                buffer[(col.x, col.y)] = fill.clone();
            }
        }

        let banner_chars: Vec<char> = BANNER.chars().collect();
        let banner_len = banner_chars.len().max(1);
        let scroll = (t * 14.0) as i32;

        let mut glyph = Cell::new(" ");
        glyph.set_fg(fg);
        glyph.set_bg(bg);

        for row_idx in 0..rect.height {
            if row_idx % 2 == 1 && !flash_on {
                continue;
            }

            let row_phase = f64::from(row_idx) * 0.65;
            let h_offset = ((t * 3.4 + row_phase).sin() * f64::from(rect.width) * 0.42) as i32;
            let y = rect.y + row_idx;
            if y >= rect.bottom() {
                break;
            }

            for x in 0..rect.width {
                let pos = (i32::from(x) + h_offset + scroll)
                    .rem_euclid(banner_len as i32) as usize;
                let ch = banner_chars[pos];
                let mut c = glyph.clone();
                let sym = ch.to_string();
                c.set_symbol(sym.as_str());
                if matches!(ch, '!' | '>' | '<') {
                    c.set_fg(bright_red);
                    c.set_bg(accent_bg);
                }
                buffer[(rect.x + x, y)] = c;
            }
        }

        if rect.height >= 3 {
            let mid = rect.y + rect.height / 2;
            let band: Vec<char> = CENTER_BAND.chars().collect();
            let band_len = band.len().max(1) as i32;
            let max_shift = i32::from(rect.width).saturating_sub(band_len) / 2;
            let center_offset =
                ((t * 4.2).sin() * f64::from(max_shift.max(0))).round() as i32 + max_shift;

            for x in 0..rect.width {
                let idx = (i32::from(x) - center_offset).rem_euclid(band_len) as usize;
                let ch = band[idx];
                let mut c = glyph.clone();
                let sym = ch.to_string();
                c.set_symbol(sym.as_str());
                c.set_fg(if flash_on { bright_red } else { dim_red });
                c.set_bg(if flash_on { accent_bg } else { bg });
                buffer[(rect.x + x, mid)] = c;
            }
        }

        Ok(())
    }
}

impl Animation for EffectError {
    fn apply_to_ve(
        &self,
        ve: &mut VisualElement,
    ) -> anyhow::Result<()> {
        Self::ensure_custom_drawer(ve);
        let t = self.run_time.as_secs_f64();
        if let VisualElementKind::Custom { drawer } = &mut ve.kind {
            drawer.draw = Box::new(move |ve, buffer, rect| Self::draw_warning(ve, buffer, rect, t));
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

impl AnyAnimation for EffectError {}
