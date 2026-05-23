use tmj_core::script::{FromCommand, IntoScriptValue, IntoTable, ScriptContext, ScriptValue, Table, TypeName, script_sym};

use crate::utils::script_args::parse_required_arg;

script_sym!(CONTENT, Member, "文本对象内容");
script_sym!(XPOS, Member, "文本对象 X 坐标");
script_sym!(YPOS, Member, "文本对象 Y 坐标");


#[derive(Default)]
#[derive(TypeName)]
pub struct TextObj {
    pub content: String,
    pub pos: (i32, i32),
}


impl IntoTable for TextObj {
    fn into_data_table(self, ctx: &mut tmj_core::script::ScriptContext) -> tmj_core::script::Table {
        let tuid = ctx.alloc_table_id();
        let mut table = Table::with_tuid(tuid);
        table.set(CONTENT, self.content.into_script_val(), None);
        table.set(XPOS, self.pos.0.into_script_val(), None);
        table.set(YPOS, self.pos.1.into_script_val(), None);
        table
    }
}

impl FromCommand for TextObj {
    fn from_script_command(
        _ctx: &mut ScriptContext,
        args: Vec<ScriptValue>,
    ) -> Result<Self, String> {
        let content = parse_required_arg(&args, 0, ScriptValue::as_string)
            .map_err(|e| e.to_string())?;
        let x = parse_required_arg(&args, 1, ScriptValue::as_int).map_err(|e| e.to_string())?;
        let y = parse_required_arg(&args, 2, ScriptValue::as_int).map_err(|e| e.to_string())?;
        Ok(TextObj {
            content,
            pos: (x.try_into().unwrap_or(0), y.try_into().unwrap_or(0)),
        })
    }
}
