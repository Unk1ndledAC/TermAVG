use std::{
    sync::{Arc, Mutex},
    time::{self, Duration},
};

use rand::Rng;
use ratatui::style::Color;
use tmj_core::script::TypeName;

use crate::pages::behaviour::{
    animation::{Animation, AnyAnimation},
    visual_element::{VisualElement, VisualElementCustomDrawer, VisualElementKind},
};

const ASCII_CHARS: &[u8] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()_+-=[]{}|;:,.<>?/~";
const TRAIL_MAX: usize = 24;

struct StreamState {
    head: f64,
    chars: Vec<char>,
    speed: f64,
}

/// 文字流动画：随机 ASCII 字符从上到下流淌，维护列级下落状态
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
        let t = self.run_time.as_secs_f64();
        if let VisualElementKind::Custom { drawer } = &mut ve.kind {
            drawer.draw = Box::new(move |_ve, buffer, rect| {
                if rect.width == 0 || rect.height == 0 {
                    return Ok(());
                }

                let mut streams = streams.lock().unwrap();

                if streams.len() != rect.width as usize {
                    let mut rng = rand::thread_rng();
                    *streams = (0..rect.width)
                        .map(|i| {
                            let len = rect.height as usize;
                            StreamState {
                                head: -(i as f64 * 0.3) - rng.gen_range(5.0..40.0),
                                chars: (0..len)
                                    .map(|_| {
                                        let idx = rng.gen_range(0..ASCII_CHARS.len());
                                        ASCII_CHARS[idx] as char
                                    })
                                    .collect(),
                                speed: 2.0 + rng.random::<f64>() * 6.0,
                            }
                        })
                        .collect();
                }

                let mut rng = rand::thread_rng();
                for s in streams.iter_mut() {
                    let head_i = s.head.floor() as i32;
                    for row in 0..rect.height {
                        let dist = head_i - row as i32;
                        if dist >= 0 && (dist as usize) < TRAIL_MAX {
                            if rng.random_bool(0.04) {
                                let idx = rng.gen_range(0..ASCII_CHARS.len());
                                s.chars[row as usize] = ASCII_CHARS[idx] as char;
                            }
                        }
                    }
                }

                for (col, s) in streams.iter().enumerate() {
                    let x = rect.x + col as u16;
                    let head_i = s.head.floor() as i32;

                    for row in 0..rect.height {
                        let y = rect.y + row;
                        let dist = head_i - row as i32;

                        if dist >= 0 && (dist as usize) < TRAIL_MAX.min(s.chars.len()) {
                            let brightness = 1.0 - dist as f64 / TRAIL_MAX as f64;
                            let b = (brightness * brightness * 255.0) as u8;
                            let cell = &mut buffer[(x, y)];
                            let ch = s.chars[row as usize];
                            cell.set_symbol(ch.encode_utf8(&mut [0u8; 4]));
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
