use std::{
    cell::RefCell,
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    rc::Rc,
};

use anyhow::Ok;
use ratatui::{
    Frame,
    crossterm::event::KeyCode,
    layout::Rect,
    style::Stylize,
    text::{Line, Span},
    widgets::{List, ListItem, ListState},
};
use regex::Regex;
use tmj_core::{
    event::handler::EventDispatcher,
};

use crate::{SETTING, art::theme::{self}};

pub const SLOT_SIZE: usize = 20;

#[derive(Debug)]
pub struct Slot {
    pub path: Option<PathBuf>,
    pub time: time::OffsetDateTime,
    pub name: String,
    pub id: u8,
}

impl Slot {
    pub fn ensure_slot_path(&mut self) -> anyhow::Result<PathBuf> {
        if self.path.is_none() {
            let safe_name: String = self.name.chars()
                .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == ' ' { c } else { '_' })
                .collect();
            let file_name = format!("{}_{}.save", self.id, safe_name);
            let mut path = SETTING.abs_save_dir()?;
            path.push(file_name);
            self.path = Some(path);
        }
        Ok(self.path.clone().unwrap())
    }
    pub fn slot_pattern() -> &'static str {
        r"^\d+_.*.save$"
    }
}

pub struct SlotManager {
    slot_selections: HashMap<u8, Slot>,
    list_state: RefCell<ListState>,
    draw_mode: SlotDrawMode,
}

#[derive(Clone, Copy)]
pub enum SlotDrawMode {
    Save,
    Load,
}

impl EventDispatcher for SlotManager {
    fn on_key(&mut self, key: &ratatui::crossterm::event::KeyEvent) {
        if key.is_release() {
            return;
        }
        match key.code {
            KeyCode::Down => { self.list_state.borrow_mut().select_next(); }
            KeyCode::Up   => { self.list_state.borrow_mut().select_previous(); }
            KeyCode::Home => { self.list_state.borrow_mut().select_first(); }
            KeyCode::End  => { self.list_state.borrow_mut().select_last(); }
            _ => {}
        }
    }
}

impl SlotManager {
    pub fn new() -> anyhow::Result<Self> {
        let abs_dir = SETTING.abs_save_dir()?;
        let slot_map = SlotManager::find_save_files(&abs_dir)?;
        let mut list_state = ListState::default();
        list_state.select_first();
        Ok(Self {
            slot_selections: slot_map,
            list_state: list_state.into(),
            draw_mode: SlotDrawMode::Save,
        })
    }

    pub fn set_draw_mode(&mut self, mode: SlotDrawMode) {
        self.draw_mode = mode;
    }

    pub fn check_any_save_slot(&self) -> bool {
        for s in self.slot_selections.values() {
            if s.path.is_some() {
                return true;
            }
        }
        return false;
    }


    pub fn get_current_slot(&mut self) -> Option<&mut Slot> {
        let pos = self.list_state.borrow_mut().selected();
        match pos {
            Some(p) => self.get_slot(p).ok(),
            None => None,
        }
    }

    pub fn get_slot(&mut self, slot_id: usize) -> anyhow::Result<&mut Slot> {
        let slot_id: u8 = slot_id as u8;
        match self.slot_selections.get_mut(&slot_id) {
            Some(_slot) => Ok(_slot),
            None => anyhow::bail!("wrong slot id".to_string()),
        }
    }

    /// 获取指定目录下一层中符合模式的文件路径列表
    fn find_save_files(dir: &Path) -> anyhow::Result<HashMap<u8, Slot>> {
        // 编译正则表达式（在函数外定义可避免重复编译）
        let re = Regex::new(Slot::slot_pattern()).expect("无效的正则表达式");
        let mut matches: HashMap<u8, Slot> = fs::read_dir(dir)?
            .filter_map(|entry| {
                let entry = match entry.ok() {
                    Some(e) => e,
                    None => return None,
                };
                let path = entry.path();
                // 只处理文件，忽略目录
                if path.is_file() {
                    if let Some(filename) = path.file_name().and_then(|s| s.to_str()) {
                        if re.is_match(filename) {
                            let file_prefix = path.file_prefix().unwrap().to_str().unwrap().to_string();
                            let id = file_prefix.splitn(2, '_').next().and_then(|s| s.parse::<u8>().ok())?;
                            let name = file_prefix.splitn(2, '_').nth(1).unwrap_or("").to_string();
                            let meta_data = std::fs::metadata(&path).ok().unwrap();
                            let modify_time: time::OffsetDateTime =
                                meta_data.modified().unwrap().into();

                            return Some(Slot {
                                path: Some(path),
                                time: modify_time,
                                name,
                                id,
                            });
                        }
                    }
                }
                None
            })
            .map(|s| (s.id, s))
            .collect();

        let now = if let Result::Ok(_now) = time::OffsetDateTime::now_local() {
            _now
        } else {
            time::OffsetDateTime::now_utc()
        };
        for slot_id in 0..SLOT_SIZE {
            let slot_id = slot_id as u8;
            let slot = matches.get(&slot_id);
            if slot.is_none() {
                matches.insert(
                    slot_id as u8,
                    Slot {
                        name: "".into(),
                        path: None,
                        id: slot_id as u8,
                        time: now.clone(),
                    },
                );
            }
        }
        Ok(matches)
    }
}

impl super::Draw for SlotManager {
    fn draw(&self, frame: &mut Frame, area: Rect) {
        let variant = match self.draw_mode {
            SlotDrawMode::Save => &theme::THEME.slot_list.save,
            SlotDrawMode::Load => &theme::THEME.slot_list.load,
        };
        let mut menu_items: Vec<ListItem> = Vec::with_capacity(SLOT_SIZE);

        for pos in 0..SLOT_SIZE {
            let pos = pos as u8;
            let slot = self.slot_selections.get(&pos);
            if slot.is_none() {
                tracing::error!("{} Slot Get Failed when render slotlist", pos);
                break;
            }
            let slot = slot.unwrap();
            let _widget = match slot.path {
                Some(_) => {
                    let text = Line::from_iter([
                        Span::from(format!("Slot {:^2} ", slot.id))
                            .bold()
                            .style(variant.slot_id),
                        Span::from(format!(
                            "{:<18} {}",
                            slot.name,
                            slot.time.truncate_to_second()
                        ))
                        .style(variant.slot_info),
                    ]);
                    text
                }
                None => {
                    let text = Line::from_iter([
                        Span::from(format!("Slot {:^2} ", pos)).bold(),
                        Span::from(format!("{:<18}", "Empty")).style(variant.empty_item),
                    ]);
                    text
                }
            };
            menu_items.push(_widget.into());
        }

        let menu_ls = List::new(menu_items)
            .highlight_symbol(">>")
            .highlight_style(theme::THEME.slot_list.selected_item);

        frame.render_stateful_widget(menu_ls, area, &mut *self.list_state.borrow_mut());
    }
}

thread_local! {
pub static SAVE_MANAGER: Rc<RefCell<SlotManager>> =
     Rc::new(RefCell::new(SlotManager::new().unwrap()));

}
