use std::time::{self, Duration};

use ratatui::{
    buffer::{Buffer, Cell},
    layout::Rect,
    style::Color,
};
use tmj_core::script::TypeName;

use crate::{
    art::{self, theme},
    pages::behaviour::{
        animation::{Animation, AnyAnimation},
        visual_element::{VisualElement, VisualElementCustomDrawer, VisualElementKind},
    },
};

/// 滚动条幅文案（循环拼接）
const BANNER: &str = "    CRITICAL SYSTEM ERROR >>> MALFUNCTION DETECTED <<<    ";
/// 相邻条幅块之间的空行数（不含边框三行本身）
const BANNER_ROW_SPACING: u16 = 4;
/// 条幅文字从左向右平移的速度（列/秒）
const BANNER_SCROLL_SPEED: f64 = 10.0;
/// 条幅上下边框字符
const BORDER_CHAR: char = '═';

/// 系统错误全屏警示动效
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
        ve.clear_before_draw = false;
    }

    /// 在指定矩形内绘制错误警示画面
    fn draw_warning(
        _ve: &VisualElement,
        buffer: &mut Buffer,
        rect: Rect,
        t: f64,
    ) -> anyhow::Result<()> {
        if rect.width == 0 || rect.height == 0 {
            return Ok(());
        }

        // --- 1. 配色：正弦调制闪烁强度，在亮红/暗红与深红底之间插值 ---
        let flash_on = ((t * 5.5).sin() + 1.0) / 2.0;
        let bright_red = Color::Rgb(255, 40, 40);
        let dim_red = Color::Rgb(120, 16, 16);
        let fg = art::blend(bright_red, dim_red, flash_on);
        let bg = art::blend(Color::Rgb(28, 0, 0), theme::BLACK, flash_on);
        let accent_bg = Color::Rgb(56, 0, 0);



        // --- 3. 条幅滚动：按时间计算水平偏移，文字匀速向右移动 ---
        let banner_chars: Vec<char> = BANNER.chars().collect();
        let banner_len = banner_chars.len().max(1);
        let scroll = (t * BANNER_SCROLL_SPEED) as i32;

        let mut glyph = Cell::new(" ");
        glyph.set_fg(fg);
        glyph.set_bg(bg);

        let mut border_cell = glyph.clone();
        let border_sym = BORDER_CHAR.to_string();
        border_cell.set_symbol(border_sym.as_str());

        // 每个条幅块占 3 行（上边框 + 文字 + 下边框），块与块之间空 BANNER_ROW_SPACING 行
        let block_stride = 3u16 + BANNER_ROW_SPACING;
        let mut block_start = 0u16;
        while block_start + 2 < rect.height {
            let top_y = rect.y + block_start;
            let text_y = top_y + 1;
            let bottom_y = top_y + 2;

            // --- 3a. 绘制条幅上下边框（整行 ═）---
            for x in 0..rect.width {
                buffer[(rect.x + x, top_y)] = border_cell.clone();
                buffer[(rect.x + x, bottom_y)] = border_cell.clone();
            }

            // --- 3b. 绘制条幅文字；标点符号高亮 ---
            for x in 0..rect.width {
                let pos = (i32::from(x) - scroll).rem_euclid(banner_len as i32) as usize;
                let ch = banner_chars[pos];
                let mut c = glyph.clone();
                let sym = ch.to_string();
                c.set_symbol(sym.as_str());
                if matches!(ch, '!' | '>' | '<') {
                    c.set_fg(bright_red);
                    c.set_bg(accent_bg);
                }
                buffer[(rect.x + x, text_y)] = c;
            }

            block_start = block_start.saturating_add(block_stride);
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
