use std::rc::Rc;
use tmj_core::{
    pathes,
    script::{IntoScriptValue, RegistableType, ScriptValue, Table, TypeName, script_sym},
};

use crate::{
    SETTING,
    pages::{behaviour::{LayerBehaviour, with_behaviour_mut, with_behaviour_mut_from_ctx_rc}, script_def::{layer, var_layer_ls}},
    utils::script_args::{parse_arg, parse_duration},
};

script_sym!(LAYER, Type, "可构造的动态图层类型");
/// 创建新的 Layer Table
#[derive(Debug, Default, TypeName)]
pub struct Layer {}

script_sym!(NAME, Member, "图层名称（唯一键）");
script_sym!(Z_DEEP, Member, "绘制深度 z_index");
script_sym!(M_VISIBLE, Member, "图层是否可见");
script_sym!(X, Member, "图层左上角 X");
script_sym!(Y, Member, "图层左上角 Y");
script_sym!(W, Member, "图层宽度");
script_sym!(H, Member, "图层高度");
script_sym!(DATA, Member, "资源/效果数据（图路径或效果类型名）");
script_sym!(LAYER_TYPE, Member, "图层类型：image 或 effect");
script_sym!(SHOW, Function, "显示图层（可带淡入时长）");
script_sym!(HIDE, Function, "隐藏图层（可带淡出时长）");
script_sym!(SET, Function, "设置图层数据（字符串参数）");

impl RegistableType for Layer {
    fn create_class_table(
        ctx: &mut tmj_core::script::ScriptContext,
        args: Vec<ScriptValue>,
    ) -> Table {
        let name = parse_arg(&args, 0, "NONAME".to_string(), ScriptValue::as_string);
        let layer_type = parse_arg(&args, 1, "image".to_string(), ScriptValue::as_string);
        let z_deep = parse_arg(&args, 2, 200, ScriptValue::as_int);
        let data_str = parse_arg(&args, 3, "".to_string(), ScriptValue::as_string);
        let x = parse_arg(&args, 4, 0, ScriptValue::as_int);
        let y = parse_arg(&args, 5, 0, ScriptValue::as_int);
        let w = parse_arg(&args, 6, SETTING.resolution.0 as i64, ScriptValue::as_int);
        let h = parse_arg(&args, 7, SETTING.resolution.1 as i64, ScriptValue::as_int);

        let root_id = ctx.alloc_table_id();
        let mut table = Table::with_tuid(root_id);
        table.set(NAME, name.into_script_val(), None);
        table.set(M_VISIBLE, false.into_script_val(), None);
        table.set(Z_DEEP, z_deep.into_script_val(), None);
        table.set(LAYER_TYPE, layer_type.clone().into_script_val(), None);
        table.set(DATA, data_str.clone().into_script_val(), None);
        table.set(X, x.into_script_val(), None);
        table.set(Y, y.into_script_val(), None);
        table.set(W, w.into_script_val(), None);
        table.set(H, h.into_script_val(), None);

        match layer_type.as_str() {
            "image" => {
                let image_path = pathes::path(&data_str);
                if !image_path.is_file() {
                    tracing::error!(
                        "image layer image source path did not exist!: {:?}",
                        image_path
                    );
                }
            }
            _ => {
                tracing::error!("character args error: wrong arg 0");
            }
        };

        let layer_ls = ctx
            .get_global_val(super::env::LAYER_LS)
            .unwrap()
            .as_table()
            .unwrap();
        var_layer_ls::VLayerLs::add_layer_ref(layer_ls, &table);
        // 加入后在每帧更新里会自动新建
        table
    }

    fn attach_table_methods(
        ctx: &tmj_core::script::ContextRef,
        table_rc: &Rc<std::cell::RefCell<Table>>,
    ) -> Result<(), String> {
        {
            let table_clone = Rc::clone(table_rc);
            table_rc.borrow_mut().set(
                SHOW,
                ScriptValue::function(SHOW, move |ctx, args| {
                    table_clone
                        .borrow_mut()
                        .set(M_VISIBLE, true.into_script_val(), None);
                    let duration = parse_duration(&args, 0, 0.2);
                    with_behaviour_mut_from_ctx_rc::<LayerBehaviour, _>(ctx, |b: &mut LayerBehaviour| {
                        b.export_show(&table_clone, duration)
                    })?;
                    Ok(ScriptValue::nil())
                }),
                Some(ctx),
            );

            let table_clone = Rc::clone(table_rc);
            table_rc.borrow_mut().set(
                SET,
                ScriptValue::function(SET, move |ctx, args| {
                    let data = parse_arg(&args, 0, String::new(), ScriptValue::as_string);
                    table_clone
                        .borrow_mut()
                        .set(DATA, data.into_script_val(), None);
                    Ok(ScriptValue::nil())
                }),
                Some(ctx),
            );

            let table_clone = Rc::clone(table_rc);
            table_rc.borrow_mut().set(
                HIDE,
                ScriptValue::function(HIDE, move |ctx, args| {
                    table_clone
                        .borrow_mut()
                        .set(M_VISIBLE, false.into_script_val(), None);
                    let duration = parse_duration(&args, 0, 0.2);
                    with_behaviour_mut_from_ctx_rc::<LayerBehaviour, _>(ctx, |b: &mut LayerBehaviour| {
                        b.export_hide(&table_clone, duration)
                    })?;
                    Ok(ScriptValue::nil())
                }),
                Some(ctx),
            );
        }
        Ok(())
    }
}

