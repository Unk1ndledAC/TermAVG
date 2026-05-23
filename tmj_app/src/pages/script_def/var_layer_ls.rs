use tmj_core::script::{ScriptValue, Table, TableRef, TypeName, script_sym};

use crate::{
    pages::script_def::{BaseVariable, layer},
    utils::script_args::parse_required_table_field,
};

script_sym!(LAYER_LS, Type, "动态图层列表全局对象");

#[derive(TypeName)]
pub struct VLayerLs;
impl VLayerLs {
    pub fn add_layer_ref(ls: TableRef, layer: &Table) -> anyhow::Result<()> {
        let key = parse_required_table_field(layer, layer::NAME, None, ScriptValue::as_string)?;
        ls.borrow_mut()
            .set(key, ScriptValue::TableHandle(layer.tuid), None);
        Ok(())
    }

    pub fn del_layer_ref(ls: TableRef, layer: &Table) -> anyhow::Result<()> {
        let key = parse_required_table_field(layer, layer::NAME, None, ScriptValue::as_string)?;
        ls.borrow_mut().remove(&key);
        Ok(())
    }
}

impl BaseVariable for VLayerLs {
    fn regist_to_ctx_impl(ctx: &mut tmj_core::script::ScriptContext) -> anyhow::Result<()> {
        ctx.set_global_table(LAYER_LS);
        Ok(())
    }
}
