use std::time::Duration;

use anyhow::Context;
use tmj_core::{
    audio::AudioOp,
    script::{Interpreter, ScriptValue, TypeName, lower_str},
};

use crate::{
    audio::{self, AUDIOM, load_audio},
    pages::script_def::BaseVariable,
};

lower_str!(BGM);
// method
lower_str!(SET);
lower_str!(STOP);

// member
lower_str!(M_SOURCE);

#[derive(TypeName)]
pub struct VBgm;

impl BaseVariable for VBgm {
    fn regist_to_ctx_impl(ctx: &mut tmj_core::script::ScriptContext) -> anyhow::Result<()> {
        ctx.set_global_table(BGM);

        let _ = ctx.set_table_member(BGM, M_SOURCE, ScriptValue::Nil);

        let _ = ctx.set_table_func(BGM, STOP, |_ctx, _agrs| {
            AUDIOM.with_borrow_mut(|a| {
                a.track_mut(&audio::Tracks::Bgm).unwrap().stop();
            });
            Ok(ScriptValue::Nil)
        });

        let _ = ctx.set_table_func(BGM, SET, |_ctx, args| {
            let path = args[0].as_str().context("!!! bgm error arg type")?;

            Interpreter::eval(
                format!(
                    "set {BGM}.{M_SOURCE} \"{}\"",
                    args.last().unwrap().as_string().unwrap()
                ),
                _ctx.clone(),
            )?;

            let source = load_audio(path).context("!!! bgm load faild")?;
            let fade_type = args
                .get(1)
                .unwrap_or(&ScriptValue::Nil)
                .as_str()
                .unwrap_or(audio::FADE_IN);

            AUDIOM.with_borrow_mut(move |a| {
                tracing::info!("bgm fading! {}", path);
                match fade_type {
                    audio::FADE_IN => {
                        a.track_mut(&audio::Tracks::Bgm).unwrap().queue_batch(vec![
                            AudioOp::fade_out(Duration::from_millis(200)),
                            AudioOp::wait(Duration::from_millis(250)),
                            AudioOp::fade_in(source, Duration::from_millis(1000)),
                        ]);
                    }
                    audio::TRANSITION => {
                        a.transition(
                            &audio::Tracks::Bgm,
                            &audio::Tracks::Bgm,
                            source,
                            Duration::from_millis(1000),
                            tmj_core::audio::FadeCurve::EaseInOut,
                        );
                    }
                    _ => {}
                }
            });

            Ok(ScriptValue::Nil)
        });

        Ok(())
    }
}
