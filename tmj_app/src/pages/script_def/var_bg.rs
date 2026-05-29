use tmj_core::script::{ScriptValue, TypeName, script_sym};

use crate::{
    pages::{
        behaviour::{BackgroundBehaviour, with_behaviour_mut_from_ctx_rc},
        script_def::BaseVariable,
    },
    utils::script_args::{parse_arg, parse_duration, parse_required_arg},
};

script_sym!(BG, Type, "背景全局对象");
script_sym!(SET, Function, "立即切换背景图");
script_sym!(TRANS_TO, Function, "渐变过渡到新背景图");
script_sym!(SHOW_EDGE, Function, "显示上下黑边遮罩");
script_sym!(HIDE_EDGE, Function, "隐藏上下黑边遮罩");
script_sym!(M_IMAGE, Member, "当前背景图路径");
script_sym!(M_IS_EDGE, Member, "是否显示黑边");

/// Bg: Background Object
#[derive(TypeName)]
pub struct VBg;

impl BaseVariable for VBg {
    fn regist_to_ctx_impl(ctx: &mut tmj_core::script::ScriptContext) -> anyhow::Result<()> {
        ctx.set_global_table(BG);

        let _ = ctx.set_table_member(BG, M_IMAGE, ScriptValue::String("".into()));
        let _ = ctx.set_table_member(BG, M_IS_EDGE, ScriptValue::Bool(true));

        let _ = ctx.set_table_func(BG, HIDE_EDGE, |ctx, args| {
            let duration_sec = parse_arg(&args, 0, 1.0, ScriptValue::to_number);
            {
                let mut c = ctx.borrow_mut();
                c.set_table_member(BG, M_IS_EDGE, ScriptValue::bool(false))
                    .map_err(|e| anyhow::anyhow!(e))?;

            }
            with_behaviour_mut_from_ctx_rc::<BackgroundBehaviour, _>(ctx, |b: &mut BackgroundBehaviour| {
                b.export_hide_edge(duration_sec);
            })?;

            Ok(ScriptValue::Nil)
        });


        let _ = ctx.set_table_func(BG, SHOW_EDGE, |ctx, args| {
            let duration_sec = parse_arg(&args, 0, 1.0, ScriptValue::to_number);
            {
                let mut c = ctx.borrow_mut();
                c.set_table_member(BG, M_IS_EDGE, ScriptValue::bool(true))
                    .map_err(|e| anyhow::anyhow!(e))?;

            }
            with_behaviour_mut_from_ctx_rc::<BackgroundBehaviour, _>(ctx, |b: &mut BackgroundBehaviour| {
                b.export_show_edge(duration_sec);
            })?;

            Ok(ScriptValue::Nil)
        });

        let _ = ctx.set_table_func(BG, SET, |ctx, args| {
            let new_img_path = parse_required_arg(&args, 0, ScriptValue::as_string)?;
            {
                let mut c = ctx.borrow_mut();
                c.set_table_member(BG, M_IMAGE, ScriptValue::String(new_img_path.clone()))
                    .map_err(|e| anyhow::anyhow!(e))?;
            }
            with_behaviour_mut_from_ctx_rc::<BackgroundBehaviour, _>(ctx, |b: &mut BackgroundBehaviour| {
                b.export_set(new_img_path);
            })?;
            Ok(ScriptValue::Nil)
        });

        let _ = ctx.set_table_func(BG, TRANS_TO, |ctx, args| {
            let new_path = parse_required_arg(&args, 0, ScriptValue::as_string)?;
            let duration_sec = parse_arg(&args, 1, 0.6, ScriptValue::to_number);

            let table = ctx
                .borrow()
                .get_global_val(BG)
                .ok_or(anyhow::anyhow!("bg not found"))?
                .as_table_or_resolve(ctx)
                .ok_or(anyhow::anyhow!("bg is not table"))?;
            {
                let mut t = table.borrow_mut();
                t.set(M_IMAGE, ScriptValue::String(new_path.clone()), None);
            }

            with_behaviour_mut_from_ctx_rc::<BackgroundBehaviour, _>(ctx, |b: &mut BackgroundBehaviour| {
                b.export_trans_to(new_path, duration_sec);
            })?;
            Ok(ScriptValue::Nil)
        });

        Ok(())
    }
}
