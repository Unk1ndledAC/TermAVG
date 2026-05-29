use std::time::Duration;

use anyhow::Context;
use rodio::Source;
use tmj_core::{
    audio::{AudioOp, AudioSource},
    script::{ContextRef, Interpreter, ScriptValue, TypeName, script_sym},
};

use crate::{
    audio::{self, AUDIOM, load_audio},
    pages::script_def::BaseVariable,
    utils::script_args,
};

script_sym!(BGM, Type, "背景音乐全局对象");
script_sym!(SET, Function, "设置并播放 BGM");
script_sym!(STOP, Function, "停止 BGM（可淡出）");
script_sym!(M_SOURCE, Member, "当前 BGM 资源路径");

#[derive(TypeName)]
pub struct VBgm;

impl VBgm {
    pub fn stop(fade_duration: Duration) {
        let fade_wait = fade_duration.saturating_add(Duration::from_millis(50));

        AUDIOM.with_borrow_mut(|a| {
            let track = a.track_mut(&audio::Tracks::Bgm).unwrap();
            if fade_duration.is_zero() {
                track.stop();
            } else {
                track.queue_batch(vec![
                    AudioOp::fade_out(fade_duration),
                    AudioOp::wait(fade_wait),
                    AudioOp::Stop,
                ]);
            }
        });
    }

    pub fn set(
        ctx: &ContextRef,
        path: &str,
        fade_type: &str,
        fade_duration: Duration,
        source_volume: f32,
    ) -> anyhow::Result<()> {
        Interpreter::eval(format!("set {BGM}.{M_SOURCE} \"{}\"", path), ctx.clone())?;

        let source = load_audio(path).context("!!! bgm load faild")?;
        let source: AudioSource = Box::new(source.amplify(source_volume));
        let fade_wait = fade_duration.saturating_add(Duration::from_millis(50));
        let path_log = path.to_string();
        let fade_type = fade_type.to_string();

        AUDIOM.with_borrow_mut(move |a| { 
            tracing::info!("bgm fading! {}", path_log);
            match fade_type.as_str() {
                audio::FADE_IN => {
                    a.track_mut(&audio::Tracks::Bgm).unwrap().queue_batch(vec![
                        AudioOp::fade_out(fade_duration),
                        AudioOp::wait(fade_wait),
                        AudioOp::fade_in(source, fade_duration),
                    ]);
                }
                audio::FADE_OUT => {
                    a.track_mut(&audio::Tracks::Bgm).unwrap().queue_batch(vec![
                        AudioOp::fade_out(fade_duration),
                        AudioOp::wait(fade_wait),
                        AudioOp::play(source, 1.0),
                    ]);
                }
                audio::TRANSITION => {
                    a.transition(
                        &audio::Tracks::Bgm,
                        &audio::Tracks::Bgm,
                        source,
                        fade_duration,
                        tmj_core::audio::FadeCurve::EaseInOut,
                    );
                }
                _ => {
                    a.track_mut(&audio::Tracks::Bgm)
                        .unwrap()
                        .queue(AudioOp::play(source, 1.0));
                }
            }
        });

        Ok(())
    }
}

impl BaseVariable for VBgm {
    fn regist_to_ctx_impl(ctx: &mut tmj_core::script::ScriptContext) -> anyhow::Result<()> {
        ctx.set_global_table(BGM);

        let _ = ctx.set_table_member(BGM, M_SOURCE, ScriptValue::Nil);

        let _ = ctx.set_table_func(BGM, STOP, |_ctx, args| {
            let fade_duration = script_args::parse_duration(&args, 0, 1.0);
            Self::stop(fade_duration);
            Ok(ScriptValue::Nil)
        });
        let _ = ctx.set_table_func(BGM, SET, |ctx, args| {
            let path = script_args::parse_required_arg(
                &args,
                0,
                ScriptValue::as_string,
            )
            .context("!!! bgm error arg type")?;
            let fade_type = script_args::parse_arg(
                &args,
                1,
                audio::FADE_IN.to_string(),
                ScriptValue::as_string,
            );
            let fade_duration = script_args::parse_duration(&args, 2, 1.0);
            let source_volume = script_args::parse_volume(&args, 3, 1.0);
            Self::set(ctx, &path, &fade_type, fade_duration, source_volume)?;
            Ok(ScriptValue::Nil)
        });

        Ok(())
    }
}
