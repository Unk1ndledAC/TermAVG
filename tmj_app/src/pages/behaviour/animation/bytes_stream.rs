use std::{
    sync::{Arc, Mutex},
    time::{self, Duration},
};

use rand::Rng;
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

const ASCII_CHARS: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()_+-=[]{}|;:,.<>?/~";
const TRAIL_MAX: usize = 20;
/// 每帧每空列生成新流的概率
const SPAWN_CHANCE: f64 = 0.08;
/// 每帧每字符突变的概率
const MUTATE_CHANCE: f64 = 0.04;

struct StreamState {
    col: u16,
    head: f64,
    speed: f64,
    chars: [char; TRAIL_MAX],
}

/// 文字流动画：随机 ASCII 字符持续随机下落，流随时间逐列生成
#[derive(TypeName, Default)]
pub struct EffectBytesStream {
    pub run_time: time::Duration,
    streams: Arc<Mutex<Vec<StreamState>>>,
}

impl EffectBytesStream {
    fn ensure_custom_drawer(ve: &mut VisualElement) {
        if !matches!(ve.kind, VisualElementKind::Custom { .. }) {
            ve.kind = VisualElementKind::Custom {
                drawer: VisualElementCustomDrawer::from(|_, _, _| Ok(())),
            };
        }
        ve.clear_before_draw = false;
    }
}

impl Animation for EffectBytesStream {
    fn apply_to_ve(&self, ve: &mut VisualElement) -> anyhow::Result<()> {
        Self::ensure_custom_drawer(ve);
        let streams = Arc::clone(&self.streams);
        if let VisualElementKind::Custom { drawer } = &mut ve.kind {
            drawer.draw = Box::new(move |_ve, buffer, rect| {
                if rect.width == 0 || rect.height == 0 {
                    return Ok(());
                }

                let mut rng = rand::thread_rng();
                let mut streams = streams.lock().unwrap();

                if streams.len() < rect.width as usize {
                    let occupied: Vec<u16> = streams.iter().map(|s| s.col).collect();
                    for _ in 0..(rect.width as usize).saturating_sub(streams.len()) {
                        let candidates: Vec<u16> = (0..rect.width)
                            .filter(|c| !occupied.contains(c))
                            .collect();
                        if candidates.is_empty() || !rng.random_bool(SPAWN_CHANCE) {
                            break;
                        }
                        let col = candidates[rng.random_range(0..candidates.len())];
                        streams.push(StreamState {
                            col,
                            head: -(rng.random::<f64>() * TRAIL_MAX as f64),
                            speed: 2.0 + rng.random::<f64>() * 5.0,
                            chars: core::array::from_fn(|_| {
                                let idx = rng.gen_range(0..ASCII_CHARS.len());
                                ASCII_CHARS[idx] as char
                            }),
                        });
                    }
                }

                let limit = rect.height as f64 + TRAIL_MAX as f64;
                streams.retain(|s| s.head < limit);

                for s in streams.iter_mut() {
                    let head_i = s.head.floor() as i32;
                    let x = rect.x + s.col;

                    for row in 0..rect.height {
                        let dist = head_i - row as i32;
                        if dist >= 0 && (dist as usize) < TRAIL_MAX {
                            let idx = dist as usize;
                            if rng.random_bool(MUTATE_CHANCE) {
                                let ci = rng.gen_range(0..ASCII_CHARS.len());
                                s.chars[idx] = ASCII_CHARS[ci] as char;
                            }
                            let brightness = 1.0 - dist as f64 / TRAIL_MAX as f64;
                            let b = (brightness * brightness * 255.0) as u8;
                            let cell = &mut buffer[(x, rect.y + row)];
                            cell.set_symbol(s.chars[idx].encode_utf8(&mut [0u8; 4]));
                            cell.set_fg(Color::Rgb(b, b / 4, b / 4));
                        }
                    }
                }

                Ok(())
            });
        }
        Ok(())
    }

    fn update(&mut self, tick_delta: Duration) {
        self.run_time += tick_delta;
        let delta = tick_delta.as_secs_f64();
        if let Ok(mut streams) = self.streams.lock() {
            for s in streams.iter_mut() {
                s.head += s.speed * delta;
            }
        }
    }

    fn force_over(&mut self) {}

    fn reset(&mut self) {
        self.run_time = Duration::ZERO;
        if let Ok(mut streams) = self.streams.lock() {
            streams.clear();
        }
    }

    fn is_animing(&self) -> bool {
        false
    }

    fn is_indeterminate(&self) -> bool {
        true
    }
}

impl AnyAnimation for EffectBytesStream {}
