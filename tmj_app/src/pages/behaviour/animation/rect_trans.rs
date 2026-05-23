use std::time::Duration;

use ratatui::layout::Rect;
use tmj_core::script::TypeName;

use crate::pages::behaviour::{
    animation::{Animation, AnyAnimation, anim_normalized_time},
    visual_element::VisualElement,
};

/// 矩形过渡的缓动曲线（`t` 为归一化时间 [0, 1]）。
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RectTransCurve {
    /// 匀速
    Linear,
    /// 缓入：`t^exponent`
    PowerIn { exponent: f64 },
    /// 缓出：`1 - (1-t)^exponent`
    PowerOut { exponent: f64 },
    /// 缓入缓出（前半段 PowerIn，后半段 PowerOut）
    PowerInOut { exponent: f64 },
    /// 正弦缓入
    SineIn,
    /// 正弦缓出
    SineOut,
    /// 正弦缓入缓出
    SineInOut,
    /// 平滑阶跃 `3t² - 2t³`
    SmoothStep,
    /// 指数缓入
    ExpIn,
    /// 指数缓出
    ExpOut,
}

impl Default for RectTransCurve {
    fn default() -> Self {
        Self::Linear
    }
}

impl RectTransCurve {
    pub fn apply(&self, t: f32) -> f32 {
        let t = t.clamp(0.0, 1.0);
        let tf = t as f64;
        let out = match self {
            RectTransCurve::Linear => tf,
            RectTransCurve::PowerIn { exponent } => tf.powf(exponent.max(0.0)),
            RectTransCurve::PowerOut { exponent } => {
                1.0 - (1.0 - tf).powf(exponent.max(0.0))
            }
            RectTransCurve::PowerInOut { exponent } => {
                let exp = exponent.max(0.0);
                if tf < 0.5 {
                    0.5 * (2.0 * tf).powf(exp)
                } else {
                    1.0 - 0.5 * (2.0 * (1.0 - tf)).powf(exp)
                }
            }
            RectTransCurve::SineIn => 1.0 - (tf * std::f64::consts::FRAC_PI_2).cos(),
            RectTransCurve::SineOut => (tf * std::f64::consts::FRAC_PI_2).sin(),
            RectTransCurve::SineInOut => {
                -( (std::f64::consts::PI * tf).cos() - 1.0) / 2.0
            }
            RectTransCurve::SmoothStep => tf * tf * (3.0 - 2.0 * tf),
            RectTransCurve::ExpIn => {
                if tf <= 0.0 {
                    0.0
                } else {
                    2.0f64.powf(10.0 * (tf - 1.0))
                }
            }
            RectTransCurve::ExpOut => {
                if tf >= 1.0 {
                    1.0
                } else {
                    1.0 - 2.0f64.powf(-10.0 * tf)
                }
            }
        };
        out.clamp(0.0, 1.0) as f32
    }
}

/// 平移方向（相对终端坐标：y 向下增大）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlideDirection {
    Up,
    Down,
    Left,
    Right,
}

/// 缩放时保持不动的锚点。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResizeAnchor {
    TopLeft,
    Top,
    TopRight,
    Left,
    Center,
    Right,
    BottomLeft,
    Bottom,
    BottomRight,
}

impl ResizeAnchor {
    fn fractions(self) -> (f64, f64) {
        match self {
            ResizeAnchor::TopLeft => (0.0, 0.0),
            ResizeAnchor::Top => (0.5, 0.0),
            ResizeAnchor::TopRight => (1.0, 0.0),
            ResizeAnchor::Left => (0.0, 0.5),
            ResizeAnchor::Center => (0.5, 0.5),
            ResizeAnchor::Right => (1.0, 0.5),
            ResizeAnchor::BottomLeft => (0.0, 1.0),
            ResizeAnchor::Bottom => (0.5, 1.0),
            ResizeAnchor::BottomRight => (1.0, 1.0),
        }
    }
}

/// 由起始矩形、平移方向与距离计算目标矩形。
pub fn slide_target(start: Rect, direction: SlideDirection, distance: u16) -> Rect {
    let (x, y) = match direction {
        SlideDirection::Up => (start.x, start.y.saturating_sub(distance)),
        SlideDirection::Down => (start.x, start.y.saturating_add(distance)),
        SlideDirection::Left => (start.x.saturating_sub(distance), start.y),
        SlideDirection::Right => (start.x.saturating_add(distance), start.y),
    };
    Rect::new(x, y, start.width, start.height)
}

/// 由起始矩形、锚点与新尺寸计算目标矩形（锚点在屏幕上的像素位置保持不变）。
pub fn resize_target(start: Rect, anchor: ResizeAnchor, new_width: u16, new_height: u16) -> Rect {
    let (ax_frac, ay_frac) = anchor.fractions();
    let ax = start.x as f64 + start.width as f64 * ax_frac;
    let ay = start.y as f64 + start.height as f64 * ay_frac;
    let x = (ax - new_width as f64 * ax_frac).round().clamp(0.0, u16::MAX as f64) as u16;
    let y = (ay - new_height as f64 * ay_frac).round().clamp(0.0, u16::MAX as f64) as u16;
    Rect::new(x, y, new_width, new_height)
}

fn lerp_u16(a: u16, b: u16, t: f32) -> u16 {
    let af = a as f64;
    let bf = b as f64;
    (af + (bf - af) * f64::from(t))
        .round()
        .clamp(0.0, u16::MAX as f64) as u16
}

fn lerp_dim(a: u16, b: u16, t: f32) -> u16 {
    let af = a as f64;
    let bf = b as f64;
    (af + (bf - af) * f64::from(t))
        .round()
        .max(0.0)
        .clamp(0.0, u16::MAX as f64) as u16
}

fn lerp_rect(start: Rect, target: Rect, eased_t: f32) -> Rect {
    Rect::new(
        lerp_u16(start.x, target.x, eased_t),
        lerp_u16(start.y, target.y, eased_t),
        lerp_dim(start.width, target.width, eased_t),
        lerp_dim(start.height, target.height, eased_t),
    )
}

/// 在 `anim_time` 内将 `VisualElement.rect` 从 `start_rect` 插值到 `target_rect`。
#[derive(TypeName)]
pub struct AniRectTrans {
    pub anim_time: Duration,
    pub start_rect: Rect,
    pub target_rect: Rect,
    pub curve: RectTransCurve,
    pub run_time: Duration,
}

impl Default for AniRectTrans {
    fn default() -> Self {
        Self {
            anim_time: Duration::ZERO,
            start_rect: Rect::default(),
            target_rect: Rect::default(),
            curve: RectTransCurve::default(),
            run_time: Duration::ZERO,
        }
    }
}

impl AniRectTrans {
    /// 开始一次矩形过渡（会重置已流逝时间）。
    pub fn begin(
        &mut self,
        start: Rect,
        target: Rect,
        duration_secs: f64,
        curve: RectTransCurve,
    ) {
        self.start_rect = start;
        self.target_rect = target;
        self.curve = curve;
        self.anim_time = Duration::from_secs_f64(duration_secs.max(0.0));
        self.run_time = Duration::ZERO;
    }

    /// 线性匀速过渡（兼容旧 API）。
    pub fn export_rect_trans(&mut self, start: Rect, target: Rect, duration_secs: f64) {
        self.begin(start, target, duration_secs, RectTransCurve::Linear);
    }

    /// 从 `start` 沿 `direction` 平移 `distance` 格，过渡到目标矩形。
    pub fn export_slide(
        &mut self,
        start: Rect,
        direction: SlideDirection,
        distance: u16,
        duration_secs: f64,
        curve: RectTransCurve,
    ) {
        let target = slide_target(start, direction, distance);
        self.begin(start, target, duration_secs, curve);
    }

    /// 以 `anchor` 为固定点，将矩形缩放到 `new_width` × `new_height`。
    pub fn export_resize(
        &mut self,
        start: Rect,
        anchor: ResizeAnchor,
        new_width: u16,
        new_height: u16,
        duration_secs: f64,
        curve: RectTransCurve,
    ) {
        let target = resize_target(start, anchor, new_width, new_height);
        self.begin(start, target, duration_secs, curve);
    }

}

impl AnyAnimation for AniRectTrans {}

impl Animation for AniRectTrans {
    fn apply_to_ve(&self, ve: &mut VisualElement) -> anyhow::Result<()> {
        let eased = self
            .curve
            .apply(anim_normalized_time(self.run_time, self.anim_time));
        ve.rect = lerp_rect(self.start_rect, self.target_rect, eased);
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
    fn slide_up_moves_y_up() {
        let start = Rect::new(10, 20, 5, 8);
        let target = slide_target(start, SlideDirection::Up, 7);
        assert_eq!(target, Rect::new(10, 13, 5, 8));
    }

    #[test]
    fn resize_from_center_keeps_center() {
        let start = Rect::new(10, 10, 20, 20);
        let target = resize_target(start, ResizeAnchor::Center, 10, 10);
        assert_eq!(target, Rect::new(15, 15, 10, 10));
    }

    #[test]
    fn curve_endpoints() {
        assert!((RectTransCurve::SineInOut.apply(0.0) - 0.0).abs() < 1e-5);
        assert!((RectTransCurve::SineInOut.apply(1.0) - 1.0).abs() < 1e-5);
    }
}
