use std::time::Duration;

use anyhow::Context;
use rodio::Source;
use tmj_core::{
    audio::{AudioOp, AudioSource},
    script::{ContextRef, ScriptValue, TypeName, script_sym},
};

use crate::{
    audio::{AUDIOM, Tracks, load_audio},
    pages::script_def::BaseVariable,
    utils::script_args,
};

script_sym!(ENV_EFFECT, Type, "环境音效全局对象");
script_sym!(SET, Function, "设置并播放环境音");
script_sym!(STOP, Function, "停止环境音（可淡出）");
script_sym!(M_SOURCE, Member, "当前环境音资源路径");
pub const SOURCE: &str = M_SOURCE;

#[derive(TypeName)]
pub struct VEnvEffect;

impl VEnvEffect {
    pub fn stop(fade_duration: Duration) {
        let fade_wait = fade_duration.saturating_add(Duration::from_millis(50));
        AUDIOM.with_borrow_mut(|a| { let a = a.as_mut().unwrap();
            if let Some(t) = a.track_mut(&Tracks::EnvEffect) {
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
    }

    pub fn set(
        ctx: &ContextRef,
        path: &str,
        fade_duration: Duration,
        source_volume: f32,
    ) -> anyhow::Result<()> {
        let fade_wait = fade_duration.saturating_add(Duration::from_millis(50));

        {
            let mut c = ctx.borrow_mut();
            if path.is_empty() {
                c.set_table_member(ENV_EFFECT, M_SOURCE, ScriptValue::Nil)
                    .map_err(|e| anyhow::anyhow!(e))?;
            } else {
                c.set_table_member(ENV_EFFECT, M_SOURCE, ScriptValue::String(path.to_string()))
                    .map_err(|e| anyhow::anyhow!(e))?;
            }
        }

        if path.is_empty() {
            Self::stop(fade_duration);
        } else {
            let source = load_audio(path)
                .map_err(|e| anyhow::anyhow!("env_effect: failed to load audio: {e}"))?;
            let source: AudioSource = Box::new(source.amplify(source_volume));
            AUDIOM.with_borrow_mut(|a| { let a = a.as_mut().unwrap();
                if let Some(t) = a.track_mut(&Tracks::EnvEffect) {
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
        }

        Ok(())
    }
}

impl BaseVariable for VEnvEffect {
    fn regist_to_ctx_impl(ctx: &mut tmj_core::script::ScriptContext) -> anyhow::Result<()> {
        ctx.set_global_table(ENV_EFFECT);
        let _ = ctx.set_table_member(ENV_EFFECT, M_SOURCE, ScriptValue::Nil);
        let _ = ctx.set_table_func(ENV_EFFECT, SET, |ctx, args| {
            let path = script_args::parse_required_arg(&args, 0, ScriptValue::as_string)
                .context("env_effect.set requires file path string")?;
            let fade_duration = script_args::parse_duration(&args, 1, 0.0);
            let source_volume = script_args::parse_volume(&args, 2, 1.0);
            Self::set(ctx, &path, fade_duration, source_volume)?;
            Ok(ScriptValue::Nil)
        });
        let _ = ctx.set_table_func(ENV_EFFECT, STOP, |ctx, args| {
            let fade_duration = script_args::parse_duration(&args, 0, 0.0);
            ctx.borrow_mut()
                .set_table_member(ENV_EFFECT, M_SOURCE, ScriptValue::string(""))
                .map_err(|s| anyhow::anyhow!(s))
                .context("stop clear m_source faild")?;
            Self::stop(fade_duration);
            Ok(ScriptValue::Nil)
        });

        Ok(())
    }
}
