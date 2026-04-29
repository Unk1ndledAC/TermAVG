mod about_content;
mod cmd;
mod gallery_ls;
mod game_setting;
mod history;
mod history_ls;
mod load;
mod save;
pub use about_content::AboutContentPopItem;
pub use game_setting::GameSettingPopItem;
pub use gallery_ls::GalleryLsPopItem;
pub use history_ls::{HISTORY_LS, DialogueRecord};
pub use history::DialogueHistoryLs;
pub use load::LoadPopItem;
pub use save::SavePopItem;

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    time::Duration,
};

pub use cmd::CmdInputItem;
use ratatui::crossterm::event::KeyEvent;
use tmj_core::event::handler::EventDispatcher;
use tmj_core::event::EventManager;


pub trait PopItem: Any + 'static {

    fn set_visual(&mut self, visual: bool);

    fn draw(&self, _frame: &mut ratatui::Frame, _rect: ratatui::layout::Rect) -> anyhow::Result<()>;

    fn is_show(&self) -> bool;

    fn is_hide(&self) -> bool {
        !self.is_show()
    }

    fn hide(&mut self){
        self.set_visual(false);
        EventManager::cool_down(Duration::from_millis(100));
    }

    fn show(&mut self){
        self.set_visual(true);
        EventManager::cool_down(Duration::from_millis(100));
    }

}

impl dyn PopItem {
    pub fn as_item<T: PopItem>(&mut self) -> Option<&mut T> {
        let any_self = self as &mut dyn Any;
        any_self.downcast_mut::<T>()
    }

    pub fn as_item_ref<T: PopItem>(&self) -> Option<&T> {
        let any_self = self as &dyn Any;
        any_self.downcast_ref::<T>()
    }
}

pub trait PopInteractiveItem: PopItem + EventDispatcher {}

impl<T> PopInteractiveItem for T where T: PopItem + EventDispatcher {}

impl dyn PopInteractiveItem {
    pub fn as_item<T: PopItem>(&mut self) -> Option<&mut T> {
        (self as &mut dyn PopItem).as_item::<T>()
    }

    pub fn as_item_ref<T: PopItem>(&self) -> Option<&T> {
        (self as &dyn PopItem).as_item_ref::<T>()
    }
}

#[derive(Default)]
pub struct PopItemStore {
    items: HashMap<TypeId, Box<dyn PopInteractiveItem>>,
    order: Vec<TypeId>,
}

impl PopItemStore {
    pub fn get_or_insert_with<T, F>(&mut self, factory: F) -> &mut T
    where
        T: PopInteractiveItem,
        F: FnOnce() -> T,
    {
        let id = TypeId::of::<T>();
        if !self.items.contains_key(&id) {
            self.items.insert(id, Box::new(factory()));
            self.order.push(id);
        }
        self.get_mut::<T>()
            .expect("pop item inserted but downcast failed unexpectedly")
    }

    pub fn get_mut<T>(&mut self) -> Option<&mut T>
    where
        T: PopInteractiveItem,
    {
        let id = TypeId::of::<T>();
        self.items
            .get_mut(&id)
            .and_then(|item| item.as_mut().as_item::<T>())
    }

    pub fn get<T>(&self) -> Option<&T>
    where
        T: PopInteractiveItem,
    {
        let id = TypeId::of::<T>();
        self.items.get(&id).and_then(|item| item.as_ref().as_item_ref::<T>())
    }

    pub fn has_visible(&self) -> bool {
        self.order
            .iter()
            .filter_map(|id| self.items.get(id))
            .any(|item| item.is_show())
    }

    pub fn draw_visible(&self, frame: &mut ratatui::Frame, area: ratatui::layout::Rect) {
        for id in &self.order {
            if let Some(item) = self.items.get(id).filter(|item| item.is_show()) {
                let _ = item.draw(frame, area);
                break;
            }
        }
    }

    pub fn dispatch_key_to_top(&mut self, key: &KeyEvent) -> bool {
        for id in self.order.iter().rev() {
            if let Some(item) = self.items.get_mut(id).filter(|item| item.is_show()) {
                item.on_key(key);
                return true;
            }
        }
        false
    }
}

