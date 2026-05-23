use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::HashMap, fs, rc::Rc};
use tmj_core::{
    pathes,
    script::{IntoScriptValue, RegistableType, ScriptValue, TabelGet, Table, TypeName, script_sym},
};

use crate::{
    pages::{
        behaviour::{CharactersStage, with_behaviour_mut_from_ctx_rc},
        pop_items::DialogueRecord,
    },
    utils::script_args::{parse_duration, parse_required_arg, parse_required_member},
};

script_sym!(CHARACTER, Type, "可构造的角色类型");
/// 创建新的 Character Table
#[derive(Serialize, Deserialize, Debug, Default, TypeName)]
pub struct Character {
    _current_face: String,
    display: String,
    stands: HashMap<String, String>,
    faces: HashMap<String, String>,
    voice: HashMap<String, String>,
    #[serde(flatten)] // 将额外字段展平到顶层
    extra: toml::Table, // 其他任意字典数据
}

script_sym!(DISPLAY, Member, "角色显示名");
script_sym!(_STANDS, Member, "立绘表（表情名 → 图片路径）");
script_sym!(_FACES, Member, "表情名列表");
script_sym!(_VOICES, Member, "语音表");
script_sym!(FACE, Member, "当前表情名");
script_sym!(SAY, Function, "角色说话（立绘、文本、语音）");
script_sym!(FADE_IN, Function, "入场：自右向左滑入 8 格并淡入到场上位置");
script_sym!(FADE_OUT, Function, "退场：淡出并从场上列表移除，其余角色平滑移位");

impl RegistableType for Character {
    fn create_class_table(
        ctx: &mut tmj_core::script::ScriptContext,
        args: Vec<ScriptValue>,
    ) -> Table {
        match parse_required_arg(&args, 0, ScriptValue::as_string) {
            Ok(setting_file) => {
                let file = pathes::path(&setting_file);
                if !file.is_file() {
                    tracing::error!("{} is not exist", &setting_file);
                    let id = ctx.alloc_table_id();
                    return Table::with_tuid(id);
                }
                let toml_str = fs::read_to_string(file).unwrap();
                let character: Character = match toml::from_str(&toml_str) {
                    Ok(res) => res,
                    Err(_info) => {
                        tracing::error!("when create character from file: {}", _info);
                        Character::default()
                    }
                };

                // 2. to table data
                let root_id = ctx.alloc_table_id();
                let mut table = Table::with_tuid(root_id);
                table.set(DISPLAY, character.display.into_script_val(), None);
                table.set(
                    _STANDS,
                    ScriptValue::Table(Rc::new(RefCell::new(Table::from_hashmap_with_tuid(
                        ctx.alloc_table_id(),
                        character.stands,
                    )))),
                    None,
                );
                table.set(
                    _FACES,
                    ScriptValue::Table(Rc::new(RefCell::new(Table::from_hashmap_with_tuid(
                        ctx.alloc_table_id(),
                        character.faces,
                    )))),
                    None,
                );
                table.set(
                    _VOICES,
                    ScriptValue::Table(Rc::new(RefCell::new(Table::from_hashmap_with_tuid(
                        ctx.alloc_table_id(),
                        character.voice,
                    )))),
                    None,
                );
                table.set(FACE, character._current_face.into_script_val(), None);
                table
            }
            Err(e) => {
                tracing::error!("character args error: {e}");
                Table::with_tuid(ctx.alloc_table_id())
            }
        }
    }

    fn attach_table_methods(
        ctx: &tmj_core::script::ContextRef,
        table_rc: &Rc<std::cell::RefCell<Table>>,
    ) -> Result<(), String> {
        {
            let table_clone = Rc::clone(table_rc);
            table_rc.borrow_mut().set(
                SAY,
                ScriptValue::function(SAY, move |ctx, args| {
                    let text = parse_required_arg(&args, 0, ScriptValue::as_string)?;
                    let speaker_name =
                        parse_required_member(&table_clone, DISPLAY, ScriptValue::as_string)?;
                    let cur_face =
                        parse_required_member(&table_clone, FACE, ScriptValue::as_string)?;
                    let faces_sv = table_clone.get(_FACES)?;
                    let face_path = ctx
                        .borrow()
                        .resolve_table_value(&faces_sv)
                        .ok()
                        .and_then(|faces_tbl| {
                            faces_tbl.borrow().get(&cur_face, None)
                        })
                        .and_then(|v| v.as_str().map(str::to_string))
                        .unwrap_or_else(|| {
                            tracing::warn!("got character face img failed; set face none");
                            String::new()
                        });

                    tracing::info!("{speaker_name} is saying {text}");

                    crate::pages::pop_items::HISTORY_LS
                        .lock()
                        .unwrap()
                        .push(DialogueRecord {
                            id: ctx.borrow().session_counter(),
                            speaker: speaker_name.clone(),
                            content: text.to_string(),
                        });

                    with_behaviour_mut_from_ctx_rc::<
                        crate::pages::behaviour::dialogue_frame::FrameBehaviour,
                        _,
                    >(ctx, |b| {
                        b.export_say(speaker_name.clone(), face_path, text.to_string());
                    })?;

                    Ok(ScriptValue::nil())
                }),
                Some(ctx),
            );
        }

        {
            let table_clone = Rc::clone(table_rc);
            table_rc.borrow_mut().set(
                FADE_IN,
                ScriptValue::function(FADE_IN, move |ctx, args| {
                    let duration = parse_duration(&args, 0, 0.6);
                    with_behaviour_mut_from_ctx_rc::<CharactersStage, _>(ctx, |b| {
                        b.export_fade_in(ctx, &table_clone, duration)
                    })?;
                    Ok(ScriptValue::nil())
                }),
                Some(ctx),
            );
        }

        {
            let table_clone = Rc::clone(table_rc);
            table_rc.borrow_mut().set(
                FADE_OUT,
                ScriptValue::function(FADE_OUT, move |ctx, args| {
                    let duration = parse_duration(&args, 0, 0.2);
                    with_behaviour_mut_from_ctx_rc::<CharactersStage, _>(ctx, |b| {
                        b.export_fade_out(ctx, &table_clone, duration)
                    })?;
                    Ok(ScriptValue::nil())
                }),
                Some(ctx),
            );
        }
        Ok(())
    }
}
