use std::time::Duration;

use rodio::Source;
use tmj_core::{
    audio::{AudioOp, AudioSource},
};

use crate::audio::{AUDIOM, Tracks, load_audio};

pub struct VVoice;

impl VVoice {
    pub fn set(path: &str, fade_duration: Duration, source_volume: f32) -> anyhow::Result<()> {
        let fade_wait = fade_duration.saturating_add(Duration::from_millis(50));

        if path.is_empty() {
            AUDIOM.with_borrow_mut(|a| { let a = a.as_mut().unwrap();
                if let Some(t) = a.track_mut(&Tracks::Voice) {
                    if fade_duration.is_zero() {
                        t.stop();
                    } else {
                        t.queue_batch(vec![
                            AudioOp::fade_out(fade_duration),
                            AudioOp::wait(fade_wait),
                            AudioOp::Stop,
                        ]);
                    }
                }
            });
            return Ok(());
        }

        let source =
            load_audio(path).map_err(|e| anyhow::anyhow!("voice: failed to load audio: {e}"))?;
        let source: AudioSource = Box::new(source.amplify(source_volume));
        AUDIOM.with_borrow_mut(|a| { let a = a.as_mut().unwrap();
            if let Some(t) = a.track_mut(&Tracks::Voice) {
                if fade_duration.is_zero() {
                    t.stop();
                    t.queue(AudioOp::play(source, 1.0));
                } else {
                    t.queue_batch(vec![
                        AudioOp::fade_out(fade_duration),
                        AudioOp::wait(fade_wait),
                        AudioOp::fade_in(source, fade_duration),
                    ]);
                }
            }
        });
        Ok(())
    }
}
