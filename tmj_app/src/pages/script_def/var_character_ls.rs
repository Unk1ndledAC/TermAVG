use tmj_core::script::{ScriptValue, TypeName, script_sym};



use crate::{

    pages::{

        behaviour::{CharactersStage, with_behaviour_mut_from_ctx_rc},

        script_def::{BaseVariable, Character},

    },

    utils::script_args::parse_duration,

};



#[derive(TypeName)]

pub struct VCharacterLs;



script_sym!(CHARACTER_LS, Type, "场上角色列表全局对象");

script_sym!(SET_CHARACTERS, Function, "设置当前出场的角色表");

script_sym!(CLEAR, Function, "清空场上角色（可带淡出时长）");



impl BaseVariable for VCharacterLs {

    fn regist_to_ctx_impl(ctx: &mut tmj_core::script::ScriptContext) -> anyhow::Result<()> {

        ctx.set_global_table(CHARACTER_LS);



        // set characters

        {

            let _ = ctx

                .set_table_func(CHARACTER_LS, SET_CHARACTERS, |ctx, args| {

                    let c_ls = ctx

                        .borrow()

                        .get_global_val(CHARACTER_LS)

                        .unwrap()

                        .as_table_or_resolve(ctx)

                        .unwrap();

                    for (idx, i) in args.iter().enumerate() {

                        let c = i

                            .as_table_or_resolve(ctx)

                            .ok_or(anyhow::anyhow!("expect table but {idx} arg is not!"))

                            .map(|i| {

                                if i.borrow().is_ins::<Character>() {

                                    Ok(i)

                                } else {

                                    Err(anyhow::anyhow!("expect character but {idx} arg is not!"))

                                }

                            })??;

                        let tuid = c.borrow().tuid;

                        c_ls

                            .borrow_mut()

                            .set_int(idx as i64, ScriptValue::table_handle(tuid));

                    }

                    Ok(ScriptValue::Table(c_ls))

                })

                .map_err(|e| anyhow::anyhow!(e))?;

        }



        {

            let _ = ctx

                .set_table_func(CHARACTER_LS, CLEAR, |ctx, args| {

                    let duration = parse_duration(&args, 0, 0.2);

                    with_behaviour_mut_from_ctx_rc::<CharactersStage, _>(ctx, |b| {

                        b.export_clear(ctx, duration)

                    })?;

                    Ok(ScriptValue::nil())

                })

                .map_err(|e| anyhow::anyhow!(e))?;

        }

        Ok(())

    }

}


