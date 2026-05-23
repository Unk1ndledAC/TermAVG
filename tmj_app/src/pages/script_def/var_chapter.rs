use tmj_core::script::{ScriptValue, TypeName, script_sym};

use crate::{
    pages::{
        behaviour::{ChapterBehaviour, with_behaviour_mut_from_ctx_rc},
        script_def::BaseVariable,
    },
    utils::script_args::{parse_duration, parse_required_arg},
};

script_sym!(CHAPTER, Type, "章节标题全局对象");
script_sym!(SHOW_TITLE, Function, "显示章节主标题");
script_sym!(SHOW_SUB_TITLE, Function, "显示章节副标题");

#[derive(TypeName)]
pub struct VChapter;

impl BaseVariable for VChapter {
    fn regist_to_ctx_impl(ctx: &mut tmj_core::script::ScriptContext) -> anyhow::Result<()> {
        ctx.set_global_table(CHAPTER);
        {
            let _ = ctx
                .set_table_func(CHAPTER, SHOW_TITLE, |ctx, args| {
                    let title = parse_required_arg(&args, 0, ScriptValue::as_string)?;
                    let duration = parse_duration(&args, 1, 1.0);

                    with_behaviour_mut_from_ctx_rc::<ChapterBehaviour, _>(ctx, |b| {
                        b.export_show_title(duration, title.to_string());
                    })?;

                    let chapter = ctx
                        .borrow()
                        .get_global_val(CHAPTER)
                        .ok_or(anyhow::anyhow!("chapter not found"))?
                        .as_table_or_resolve(ctx)
                        .ok_or(anyhow::anyhow!("chapter is not table"))?;
                    Ok(ScriptValue::Table(chapter))
                })
                .map_err(|e| anyhow::anyhow!(e))?;
        }

        {
            let _ = ctx
                .set_table_func(CHAPTER, SHOW_SUB_TITLE, |ctx, args| {
                    let subtitle = parse_required_arg(&args, 0, ScriptValue::as_string)?;
                    let duration = parse_duration(&args, 1, 1.0);

                    with_behaviour_mut_from_ctx_rc::<ChapterBehaviour, _>(ctx, |b| {
                        b.export_show_sub_title(duration, subtitle.to_string());
                    })?;

                    let chapter = ctx
                        .borrow()
                        .get_global_val(CHAPTER)
                        .ok_or(anyhow::anyhow!("chapter not found"))?
                        .as_table_or_resolve(ctx)
                        .ok_or(anyhow::anyhow!("chapter is not table"))?;
                    Ok(ScriptValue::Table(chapter))
                })
                .map_err(|e| anyhow::anyhow!(e))?;
        }

        Ok(())
    }
}
