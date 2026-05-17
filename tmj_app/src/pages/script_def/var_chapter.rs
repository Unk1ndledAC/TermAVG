use std::time::Duration;

use tmj_core::script::{ScriptValue, TypeName, lower_str};

use crate::pages::{
    behaviour::{ChapterBehaviour, with_behaviour_mut_from_ctx_rc},
    script_def::BaseVariable,
};

// name
lower_str!(CHAPTER);

//method
lower_str!(SHOW_TITLE);
lower_str!(SHOW_SUB_TITLE);

#[derive(TypeName)]
pub struct VChapter;

impl BaseVariable for VChapter {
    fn regist_to_ctx_impl(ctx: &mut tmj_core::script::ScriptContext) -> anyhow::Result<()> {
        ctx.set_global_table(CHAPTER);
        {
            let _ = ctx
                .set_table_func(CHAPTER, SHOW_TITLE, |ctx, args| {
                    let title = args
                        .first()
                        .and_then(|v| v.as_str())
                        .ok_or(anyhow::anyhow!(
                            "chapter.show_title requires title string argument"
                        ))?;
                    let duration = args
                        .get(1)
                        .and_then(|v| v.to_number())
                        .filter(|d| d.is_finite() && *d >= 0.0)
                        .unwrap_or(1.0);

                    with_behaviour_mut_from_ctx_rc::<ChapterBehaviour, _>(ctx, |b| {
                        b.export_show_title(Duration::from_secs_f64(duration), title.to_string());
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
                    let subtitle = args
                        .first()
                        .and_then(|v| v.as_str())
                        .ok_or(anyhow::anyhow!(
                            "chapter.show_sub_title requires subtitle string argument"
                        ))?;
                    let duration = args
                        .get(1)
                        .and_then(|v| v.to_number())
                        .filter(|d| d.is_finite() && *d >= 0.0)
                        .unwrap_or(1.0);

                    with_behaviour_mut_from_ctx_rc::<ChapterBehaviour, _>(ctx, |b| {
                        b.export_show_sub_title(
                            Duration::from_secs_f64(duration),
                            subtitle.to_string(),
                        );
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
