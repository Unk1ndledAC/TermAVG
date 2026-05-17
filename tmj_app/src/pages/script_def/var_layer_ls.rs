use tmj_core::script::{ScriptValue, TabelGet, Table, TableRef, TypeName, lower_str};

use crate::pages::script_def::{BaseVariable, layer};

lower_str!(LAYER_LS);

#[derive(TypeName)]
pub struct VLayerLs;
impl VLayerLs {
    pub fn add_layer_ref(ls: TableRef, layer: &Table) -> anyhow::Result<()> {
        let key = layer
            .get(layer::NAME, None)
            .ok_or(anyhow::anyhow!("layer should has name filed"))?
            .as_string()
            .unwrap();
        ls.borrow_mut()
            .set(key, ScriptValue::TableHandle(layer.tuid), None);
        Ok(())
    }

    pub fn del_layer_ref(ls: TableRef, layer: &Table) -> anyhow::Result<()> {
        let key = layer
            .get(layer::NAME, None)
            .ok_or(anyhow::anyhow!("layer should has name filed"))?
            .as_string()
            .unwrap();
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
