use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::LazyLock;

use anyhow::Result;
use ratatui::Frame;
use ratatui::crossterm::event::KeyCode;
use ratatui::style::Style;
use ratatui::widgets::ListState;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span},
    widgets::{List, ListItem},
};
use strum_macros::{Display, EnumString};
use tmj_core::command::{CmdBuffer, GameCmd};
use tmj_core::event::handler::EventDispatcher;
use tmj_core::img::shape::Pic;
use tmj_core::pathes;

use crate::art::{self, theme};
use crate::pages::pipeline::{
    logical_area,
    visual_element::{VisualElement, VisualElementKind},
};
use crate::pages::pop_items::{LoadPopItem, PopItem, PopItemStore};
use crate::pages::{SAVE_MANAGER, Screen, UserScreen};
use crate::{LAYOUT, SETTING};

#[warn(dead_code)]
#[derive(Display, EnumString, Debug, PartialEq)]
enum MainSelections {
    Continue,
    Load,
    NewGame,
    Gallery,
    Setting,
    About,
    Exit,
}

const SELECTION_LEN: usize = 7;
static MAINMENU_TITLE_TEXT: LazyLock<Option<String>> = LazyLock::new(load_title_text_from_setting);

fn draw_shortkey_bar(frame: &mut Frame, area: Rect) {
    let key_style = theme::THEME.key_binding.key;
    let desc_style = theme::THEME.key_binding.description;
    let line = Line::from(vec![
        Span::styled(" ↑ ", key_style),
        Span::styled("上移 ", desc_style),
        Span::styled(" ↓ ", key_style),
        Span::styled("下移 ", desc_style),
        Span::styled(" Enter ", key_style),
        Span::styled("确认", desc_style),
    ])
    .centered();
    frame.render_widget(line, area);
}

pub struct MainScreen {
    selections: [MainSelections; SELECTION_LEN],
    select_state: RefCell<ListState>,
    pop_items: PopItemStore,
    dark_ve: VisualElement,
    bg_img_path: Option<PathBuf>,
    frame_count: usize,
}
impl Screen for MainScreen {
    fn active(&mut self, _named_args: &crate::gameflow::NamedArgs) -> anyhow::Result<super::ScreenActRespond> {
        self.frame_count = 0;
        self.bg_img_path = Self::resolve_mainmenu_bg_img();
        Ok(super::ScreenActRespond::default())
    }
}

impl super::Draw for MainScreen {
    fn draw(&self, frame: &mut Frame, area: Rect) {
        if let Some(path) = &self.bg_img_path && let Ok(bg_img) = Pic::from(path) {
            frame.render_widget(bg_img, area);
        }

        let max_x = area.x.saturating_add(area.width.saturating_sub(1));
        let menu_x = area.x.saturating_add(LAYOUT.mainmenu_lw.0).min(max_x);
        let available_w = area.width.saturating_sub(menu_x.saturating_sub(area.x));
        let menu_w = available_w.min(LAYOUT.mainmenu_lw.1).max(1);
        let menu_rect = Rect::new(menu_x, area.y, menu_w, area.height);
        frame.buffer_mut().set_style(menu_rect, theme::THEME.main_menu.block);

        let [title_rect, _, list_rect, shortkey_rect] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(28),
                Constraint::Percentage(6),
                Constraint::Percentage(60),
                Constraint::Length(1),
            ])
            .areas(menu_rect);

        art::effect::text(
            self.frame_count,
            title_rect,
            frame.buffer_mut(),
            theme::LTY_BLUE,
            MAINMENU_TITLE_TEXT.as_deref(),
        );

        let list_rect = list_rect.centered(Constraint::Length(menu_w.saturating_sub(4).max(1)), Constraint::Percentage(100));

        let mut menu_items: Vec<ListItem> = Vec::with_capacity(SELECTION_LEN);
        for (_pos, selection) in self.selections.iter().enumerate() {
            let item = ListItem::new(Line::from(Span::from(format!(
                "{:<25}",
                selection.to_string()
            ))));

            let item = match selection {
                MainSelections::Continue => {
                    if Self::has_temp_save() {
                        item.style(theme::THEME.main_menu.item)
                    } else {
                        item.style(theme::THEME.main_menu.disabled_item)
                    }
                }
                MainSelections::Load => {
                    if SAVE_MANAGER.with(|m| !m.borrow().check_any_save_slot()) {
                        item.style(theme::THEME.main_menu.disabled_item)
                    } else {
                        item.style(theme::THEME.main_menu.item)
                    }
                }
                _ => {
                    item.style(theme::THEME.main_menu.item)
                }
            };
            menu_items.push(item);
        }
        
        let menu_ls = List::new(menu_items)
            .highlight_style(theme::THEME.main_menu.selected_item)
            .highlight_symbol(">> ");
        
        frame.render_stateful_widget(menu_ls, list_rect, &mut *self.select_state.borrow_mut());
        draw_shortkey_bar(frame, shortkey_rect);
        self.draw_menu_mask_if_pop_visible(frame, menu_rect);
        self.draw_popitems(frame, area);
    }
}

impl MainScreen {
    fn draw_menu_mask_if_pop_visible(&self, frame: &mut Frame, menu_rect: Rect) {
        if self.pop_items.has_visible() {
            let _ = self.dark_ve.render(frame.buffer_mut(), menu_rect);
        }
    }

    fn draw_popitems(&self, frame: &mut Frame, area: Rect) {
        let max_x = area.x.saturating_add(area.width.saturating_sub(1));
        let pop_x = area
            .x
            .saturating_add(LAYOUT.mainmenu_load_pop_lw.0)
            .min(max_x);
        let pop_avail_w = area.width.saturating_sub(pop_x.saturating_sub(area.x));
        let configured_pop_w = if LAYOUT.mainmenu_load_pop_lw.1 == 0 {
            pop_avail_w
        } else {
            LAYOUT.mainmenu_load_pop_lw.1
        };
        let pop_w = configured_pop_w.min(pop_avail_w).max(1);
        let pop_rect = Rect::new(pop_x, area.y, pop_w, area.height);
        self.pop_items.draw_visible(frame, pop_rect);
    }

    fn has_temp_save() -> bool {
        let mut path = match SETTING.abs_save_dir() {
            std::result::Result::Ok(path) => path,
            Err(_) => return false,
        };
        path.push("temp.save");
        path.is_file()
    }

    pub fn spawn(_name_args: std::collections::HashMap<&str, &str>) -> Self {
        let mut select_state = ListState::default();
        select_state.select(Some(2));
        let select_state = RefCell::new(select_state);
        MainScreen {
            selections: [
                MainSelections::Continue,
                MainSelections::Load,
                MainSelections::NewGame,
                MainSelections::Gallery,
                MainSelections::Setting,
                MainSelections::About,
                MainSelections::Exit,
            ],
            select_state,
            pop_items: PopItemStore::default(),
            dark_ve: VisualElement {
                name: "_".into(),
                alpha: 0.4,
                style: Style::new().bg(theme::BLACK),
                rect: logical_area(),
                fill_before_draw: true,
                kind: VisualElementKind::Text { content: "".into() },
                ..Default::default()
            },
            bg_img_path: None,
            frame_count: 0,
        }
    }
}

#[derive(serde::Deserialize)]
struct DialogueSceneSaveLite {
    session_id: usize,
}

fn load_title_text_from_setting() -> Option<String> {
    let rel_path = SETTING.mainmenu_title_file.as_ref()?;
    let abs_path = pathes::path(rel_path);
    match std::fs::read_to_string(&abs_path) {
        Ok(content) => {
            let text = content.trim();
            if text.is_empty() {
                None
            } else {
                Some(text.to_string())
            }
        }
        Err(e) => {
            tracing::warn!("read mainmenu_title_file failed {:?}: {:?}", abs_path, e);
            None
        }
    }
}

impl MainScreen {
    fn temp_save_session_id() -> Option<usize> {
        let mut path = SETTING.abs_save_dir().ok()?;
        path.push("temp.save");
        let save_str = std::fs::read_to_string(path).ok()?;
        let save = json5::from_str::<DialogueSceneSaveLite>(&save_str).ok()?;
        Some(save.session_id)
    }

    fn resolve_mainmenu_bg_img() -> Option<PathBuf> {
        let default_path = pathes::path(&SETTING.mainmenu_default_bg_img);
        let session_id = Self::temp_save_session_id();

        if let Some(session_id) = session_id {
            if let Some(map_item) = SETTING.mainmenu_session_bg_map.iter().find(|item| {
                session_id >= item.session_id_min && session_id <= item.session_id_max
            }) {
                let mapped = pathes::path(&map_item.bg_img);
                if mapped.is_file() {
                    return Some(mapped);
                }
            }
        }

        if default_path.is_file() {
            return Some(default_path);
        }
        None
    }

    pub fn execute_selection(&mut self) -> Result<()> {
        let cur_selection = &self.selections[self.select_state.borrow().selected().unwrap()];
        match cur_selection {
            MainSelections::NewGame => {
                CmdBuffer::push(GameCmd::NewGame);
            }
            MainSelections::Load => {
                if SAVE_MANAGER.with(|m| m.borrow().check_any_save_slot()) {
                    self.pop_items
                        .get_or_insert_with(LoadPopItem::new_for_mainmenu)
                        .show();
                }
            }
            MainSelections::Gallery => {
                CmdBuffer::push(GameCmd::GoScene(UserScreen::Gallery.to_string()));
            }
            MainSelections::Setting => {
                CmdBuffer::push(GameCmd::GoScene(UserScreen::Setting.to_string()));
            }
            MainSelections::Exit => {
                CmdBuffer::push(GameCmd::QuitGame);
            }
            MainSelections::About => {
                CmdBuffer::push(GameCmd::GoScene(UserScreen::About.to_string()));
            }
            MainSelections::Continue => {
                if Self::has_temp_save() {
                    CmdBuffer::push(GameCmd::ContinueGame);
                }
            }
        }
        Ok(())
    }
}

impl EventDispatcher for MainScreen {
    fn handle_tick(&mut self, _tick: std::time::Duration) {
        self.frame_count += 1;
    }

    fn on_key(&mut self, key: &ratatui::crossterm::event::KeyEvent) {
        if self.pop_items.dispatch_key_to_top(key) {
            return;
        }
        if !key.is_release() {
            return;
        }
        match key.code {
            KeyCode::Down => {
                self.select_state.borrow_mut().select_next();
            }
            KeyCode::Up => {
                self.select_state.borrow_mut().select_previous();
            }
            KeyCode::Enter => {
                let _ = self.execute_selection();
            }
            _ => {}
        }
    }
}
