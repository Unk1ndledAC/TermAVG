use anyhow::Context;
use tmj_core::{
    pathes,
    script::{ContextRef, IntoScriptValue, ScriptContext, ScriptValue, lower_str},
};

use crate::{
    pages::{
        behaviour::{FrameBehaviour, with_behaviour_mut_from_ctx_rc},
        script_def::{
            BaseVariable, Character, TextObj, VBg, VBgm, VChapter, VCharacterLs, VEnvEffect,
            VFrame, VLayerLs, VParagraph, VVoice, var_frame, var_layer_ls,
        },
    },
    utils::script_args::{self, parse_arg, parse_required_arg},
};

macro_rules! script_str {
    ($ctx:ident, $name:ident) => {
        $ctx.set_global_val($name, ScriptValue::String($name.to_string()));
    }; // 两个参数：ctx, name -> 值 = name 变量的字符串值
    // 三个参数：ctx, name, value -> 值 = value 转换为 String
    ($ctx:expr, $name:ident, $value:expr) => {
        $ctx.set_global_val($name, ScriptValue::String(Into::<String>::into($value)));
    };
}

// global member
lower_str!(BGIMG_PATH);
lower_str!(BEHAVIOURS_MAP);
pub use super::var_bg::BG;
pub use super::var_bgm::BGM;
pub use super::var_chapter::CHAPTER;
pub use super::var_character_ls::CHARACTER_LS;
pub use super::var_env_effect::ENV_EFFECT;
pub use super::var_frame::FRAME;
pub use super::var_layer_ls::LAYER_LS;
pub use super::var_paragraph::PARAGRAPH;

// global function
lower_str!(TEXT);
lower_str!(DISPLAY_NAME);
lower_str!(SAVE_TO);
lower_str!(ADD_LAYER);
lower_str!(DEL_LAYER);
lower_str!(SEE);
lower_str!(VOICE);
lower_str!(LOG);

fn regist_base_gvar(ctx: &mut ScriptContext) -> anyhow::Result<()> {
    VCharacterLs::regist_to_ctx(ctx)?;
    VFrame::regist_to_ctx(ctx)?;
    VParagraph::regist_to_ctx(ctx)?;
    VLayerLs::regist_to_ctx(ctx)?;
    VBgm::regist_to_ctx(ctx)?;
    VEnvEffect::regist_to_ctx(ctx)?;
    VChapter::regist_to_ctx(ctx)?;
    VBg::regist_to_ctx(ctx)?;
    Ok(())
}

pub fn init_env(ctx: ContextRef, behaviours: crate::pages::behaviour::BehaviourMap) {
    {
        ctx.borrow_mut()
            .set_global_val(DISPLAY_NAME, ScriptValue::string(""));
    }

    let mut ctx = ctx.borrow_mut();
    {
        use crate::audio::*;
        script_str!(ctx, FADE_IN);
        script_str!(ctx, FADE_OUT);
        script_str!(ctx, TRANSITION);
        ctx.set_global_val(BEHAVIOURS_MAP, ScriptValue::rust_object(behaviours));
    }
    {
        ctx.type_registry.register::<Character>();
        ctx.type_registry.register::<TextObj>();
    }
    let _ = regist_base_gvar(&mut ctx);
    {
        ctx.set_global_func(SAVE_TO, |c, args| {
            let table = args[0]
                .as_table_or_resolve(c)
                .ok_or(anyhow::anyhow!("args 0 is not a table or handle"))?;
            let target_path = args[1]
                .as_string()
                .ok_or(anyhow::anyhow!("args 1 is not str"))?;
            let ct = toml::to_string(&table.into_script_val())?;
            let target_path = pathes::path(target_path);
            std::fs::write(target_path, ct)?;
            Ok(ScriptValue::Nil)
        });
    }

    {
        ctx.set_global_func(TEXT, |c, args| {
            let raw_text = parse_required_arg(&args, 0, &ScriptValue::as_string)
                .context("text requires content string")?;
            let mut speed = parse_arg(&args, 1, -1.0, &ScriptValue::to_number);
            let speaker = parse_arg(&args, 1, "".to_string(), &ScriptValue::as_string);
            // 第二参数可以是speak 或者speed, 如同时需要,speed 放第三位
            if speed < 0_f64 {
                if !speaker.is_empty() {
                    speed = parse_arg(&args, 2, 30.0, &ScriptValue::to_number);
                } else {
                    speed = 30.0;
                }
            }
            with_behaviour_mut_from_ctx_rc::<FrameBehaviour, _>(c, |b| {
                b.export_text(raw_text, speed, speaker);
            })?;

            Ok(ScriptValue::Nil)
        });
    }

    {
        ctx.set_global_func("create_default_character", |_ctx, args| {
            let path = args[0].as_string().unwrap();
            let character = Character::default();
            let ct = toml::to_string(&character)?;
            let path = pathes::path(path);
            let _ = std::fs::write(path, ct)?;
            Ok(ScriptValue::Nil)
        })
    }

    {
        ctx.set_global_func(SEE, |_ctx, args| {
            let name = args
                .first()
                .and_then(|x| x.as_str())
                .ok_or(anyhow::anyhow!("see requires visual element name string"))?;
            crate::pages::dialogue::see_visual_element(name)?;
            Ok(ScriptValue::Nil)
        });
    }

    {
        ctx.set_global_func(VOICE, |_ctx, args| {
            let path = script_args::parse_required_arg(&args, 0, ScriptValue::as_string)
                .context("voice requires audio file path string")?;
            let fade_duration = script_args::parse_duration(&args, 1, 0.0);
            let source_volume = script_args::parse_volume(&args, 2, 1.0);
            VVoice::set(&path, fade_duration, source_volume)?;
            Ok(ScriptValue::Nil)
        });
    }

    {
        ctx.set_global_func(LOG, |c, args| {
            let path = args
                .first()
                .ok_or(anyhow::anyhow!("log requires path argument"))?;
            let path = path
                .as_expression()
                .or_else(|| path.as_str().map(|x| x.to_string()))
                .ok_or(anyhow::anyhow!(
                    "log arg should be expression or string path"
                ))?;

            let value = c
                .borrow()
                .resolve_path(&path)
                .map_err(|e| anyhow::anyhow!(e))?;
            let message = format!("log {path} => {:?}", value);
            println!("{message}");
            tracing::info!("{message}");
            Ok(ScriptValue::Nil)
        });
    }
}
