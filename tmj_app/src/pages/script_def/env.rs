use std::path::PathBuf;

use anyhow::Context;
use tmj_core::{
    pathes,
    script::{ContextRef, IntoScriptValue, ScriptContext, ScriptValue, script_sym},
};

use crate::{
    setting::SETTING,
    utils::preparse_script,
    pages::{
        behaviour::{FrameBehaviour, with_behaviour_mut_from_ctx_rc},
        script_def::{
            BaseVariable, Character, Layer, TextObj, VBg, VBgm, VChapter, VCharacterLs, VEnvEffect, VFrame, VLayerLs, VParagraph, VVoice, var_frame, var_layer_ls
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

script_sym!(BGIMG_PATH, Member, "默认背景图路径（全局字符串）");
script_sym!(BEHAVIOURS_MAP, Member, "对话场景 Behaviour 映射表");
pub use super::var_bg::BG;
pub use super::var_bgm::BGM;
pub use super::var_chapter::CHAPTER;
pub use super::var_character_ls::CHARACTER_LS;
pub use super::var_env_effect::ENV_EFFECT;
pub use super::var_frame::FRAME;
pub use super::var_layer_ls::LAYER_LS;
pub use super::var_paragraph::PARAGRAPH;

script_sym!(TEXT, Function, "在对话框显示文本");
script_sym!(DISPLAY_NAME, Function, "显示名称标签");
script_sym!(SAVE_TO, Function, "保存游戏到槽位");
script_sym!(ADD_LAYER, Function, "向 layer_ls 添加图层");
script_sym!(DEL_LAYER, Function, "从 layer_ls 移除图层");
script_sym!(SEE, Function, "注视/看向效果");
script_sym!(VOICE, Function, "播放语音");
script_sym!(LOG, Function, "写入日志");
script_sym!(REBUILD, Function, "重新预处理 setting 指定的 fs 脚本");

/// 重新预处理 `setting.toml` 中 `preprogress_script` 列出的 fs 脚本（生成对应 fss）。
pub fn rebuild_preprogress_scripts() -> anyhow::Result<()> {
    for origin_script in &SETTING.preprogress_script {
        let o_path = pathes::path(origin_script);
        let t_path = PathBuf::from("resource")
            .join(PathBuf::from(o_path.file_name().unwrap()).with_extension("fss"));
        preparse_script(&o_path, &t_path, None)
            .with_context(|| format!("rebuild failed: {:?}", o_path))?;
        tracing::info!("rebuild script {:?} -> {:?}", o_path, t_path);
    }
    Ok(())
}

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
        ctx.type_registry.register::<Layer>();
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

    {
        ctx.set_global_func(REBUILD, |_ctx, _args| {
            rebuild_preprogress_scripts()?;
            Ok(ScriptValue::Nil)
        });
    }
}
