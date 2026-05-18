use tmj_core::script::{ScriptValue, TypeName, script_sym};


use crate::pages::script_def::BaseVariable;

script_sym!(FRAME, Type, "对话框全局对象");
script_sym!(VISIBLE, Member, "对话框是否可见");
script_sym!(MODE, Member, "对话框模式（如 normal）");
script_sym!(SPEAKER, Member, "当前说话人名称");
script_sym!(SHOW, Function, "显示对话框");
script_sym!(HIDE, Function, "隐藏对话框");
script_sym!(SET_MODE, Function, "设置对话框模式");

#[derive(TypeName)]
pub struct VFrame;

impl BaseVariable for VFrame {
    fn regist_to_ctx_impl(ctx: &mut tmj_core::script::ScriptContext) -> anyhow::Result<()> {
        ctx.set_global_table(FRAME);
        let _ = ctx.set_table_member(FRAME, VISIBLE, ScriptValue::bool(true));
        let _ = ctx.set_table_member(FRAME, MODE, ScriptValue::string("normal"));
        let _ = ctx.set_table_member(FRAME, SPEAKER, ScriptValue::string(""));

        {
            let _ = ctx
                .set_table_func(FRAME, SHOW, |ctx, _args| {
                    let frame = ctx
                        .borrow()
                        .get_global_val(FRAME)
                        .ok_or(anyhow::anyhow!("frame not found"))?
                        .as_table_or_resolve(ctx)
                        .ok_or(anyhow::anyhow!("frame is not table"))?;
                    frame.borrow_mut().set(VISIBLE, ScriptValue::bool(true), None);
                    Ok(ScriptValue::Table(frame))
                })
                .map_err(|e| anyhow::anyhow!(e))?;
        }

        {
            let _ = ctx
                .set_table_func(FRAME, HIDE, |ctx, _args| {
                    let frame = ctx
                        .borrow()
                        .get_global_val(FRAME)
                        .ok_or(anyhow::anyhow!("frame not found"))?
                        .as_table_or_resolve(ctx)
                        .ok_or(anyhow::anyhow!("frame is not table"))?;
                    frame.borrow_mut().set(VISIBLE, ScriptValue::bool(false), None);
                    Ok(ScriptValue::Table(frame))
                })
                .map_err(|e| anyhow::anyhow!(e))?;
        }

        {
            let _ = ctx
                .set_table_func(FRAME, SET_MODE, |ctx, args| {
                    let mode = args
                        .first()
                        .and_then(|x| x.as_str())
                        .ok_or(anyhow::anyhow!("frame.set_mode requires mode string"))?;
                    let frame = ctx
                        .borrow()
                        .get_global_val(FRAME)
                        .ok_or(anyhow::anyhow!("frame not found"))?
                        .as_table_or_resolve(ctx)
                        .ok_or(anyhow::anyhow!("frame is not table"))?;
                    frame.borrow_mut().set(MODE, ScriptValue::string(mode), None);
                    Ok(ScriptValue::Table(frame))
                })
                .map_err(|e| anyhow::anyhow!(e))?;
        }
        Ok(())
    }
}
