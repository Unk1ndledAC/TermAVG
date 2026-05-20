use anyhow::{Context, Ok, Result};
use ratatui::Frame;
use std::cell::RefCell;
use std::path::PathBuf;
use std::time::Duration;
use tmj_core::audio::TrackConfig;
use tmj_core::pathes;
use tmj_core::{
    command::GameCmd,
    event::{GameEvent, handler::EventDispatcher, sender::EventSender},
};
use tracing::info;

use crate::art::theme;
use crate::audio::AUDIOM;
use crate::audio::Tracks;
use crate::gameflow::GameFlowMgr;
use crate::pages::dialogue::DialogueScene;
use crate::pages::{SAVE_MANAGER, UserScreen};
use crate::utils::write_script_sym_reference;
use crate::{GAME_SETTING, SETTING};

pub struct Game {
    pub game_flow: RefCell<GameFlowMgr>,
}

impl Game {
    fn with_dialogue_mut<R>(
        &mut self,
        handler: impl FnOnce(&mut DialogueScene) -> anyhow::Result<R>,
    ) -> anyhow::Result<R> {
        let screen = self
            .game_flow
            .borrow_mut()
            .ensure(UserScreen::Dialogue.to_string())?;
        let mut screen = screen.borrow_mut();
        let dialogue = screen.as_screen::<DialogueScene>().unwrap();
        handler(dialogue)
    }

    fn temp_save_path() -> anyhow::Result<PathBuf> {
        let mut path = SETTING.abs_save_dir()?;
        path.push("temp.save");
        Ok(path)
    }

    fn save_dialogue_to_path(&mut self, target_path: PathBuf) -> anyhow::Result<()> {
        let screen = match self
            .game_flow
            .borrow_mut()
            .get_scene(&UserScreen::Dialogue.to_string())
        {
            Some(_screen) => _screen,
            None => anyhow::bail!("No Dialouge Screen"),
        };
        let mut screen = screen.borrow_mut();
        let screen = screen.as_screen::<DialogueScene>().unwrap();
        let save_str = screen.save_to()?;
        std::fs::write(target_path, save_str)?;
        Ok(())
    }

    pub fn new() -> Game {
        // 初始化音频轨道
        AUDIOM.with_borrow_mut(|a| {
            a.create_track(
                Tracks::Bgm,
                Tracks::Bgm.to_string(),
                TrackConfig {
                    looped: true,
                    default_fade_duration: Duration::from_millis(400),
                    ..Default::default()
                },
            );
            a.create_track(
                Tracks::Voice,
                Tracks::Voice.to_string(),
                TrackConfig {
                    looped: false,
                    default_fade_duration: Duration::from_millis(10),
                    ..Default::default()
                },
            );
            a.create_track(
                Tracks::EnvEffect,
                Tracks::EnvEffect.to_string(),
                TrackConfig {
                    looped: true,
                    default_fade_duration: Duration::from_millis(200),
                    ..Default::default()
                },
            );
            a.create_track(
                Tracks::MainMenuBgm,
                Tracks::MainMenuBgm.to_string(),
                TrackConfig {
                    looped: true,
                    default_fade_duration: Duration::from_millis(800),
                    ..Default::default()
                },
            );
        });

        GAME_SETTING.with_borrow(|setting| {
            if let Err(e) = setting.apply_setting() {
                tracing::warn!("apply game setting failed: {:?}", e);
            }
        });

        let mut gameflow = GameFlowMgr::new();
        let _ = gameflow
            .ensure(UserScreen::Main.to_string())
            .inspect_err(|e| tracing::error!("{:?}", e));

        let _ = gameflow
            .go_screen(&UserScreen::Main.to_string())
            .inspect_err(|e| tracing::error!("Game Main Sceen Set Failded! Game Init Failed!: {e}"));

        if let Err(e) =
            write_script_sym_reference(&pathes::path("script_env.txt"))
        {
            tracing::warn!("write script_env.txt failed: {e:?}");
        }

        if let Err(e) = crate::pages::behaviour::ve_z_index::write_ve_z_index_reference(
            &pathes::path("ve_z_index.txt"),
        ) {
            tracing::warn!("write ve_z_index.txt failed: {e:?}");
        }

        if let Err(e) = crate::pages::script_def::env::rebuild_preprogress_scripts() {
            tracing::error!("preprocess script failed: {:?}", e);
        }

        Game {
            game_flow: RefCell::new(gameflow),
        }
    }

    fn on_save_slot(&mut self, id: u8) -> anyhow::Result<()> {
        let binding = SAVE_MANAGER.with(|m| m.clone());
        let mut binding = binding.borrow_mut();
        let slot = binding.get_slot(id.into())?;
        let _ = slot.ensure_slot_path();
        tracing::info!("save slot path {:?}", slot.path);
        if slot.path.is_some() {
            self.save_dialogue_to_path(slot.path.clone().unwrap())?;
        } else {
            anyhow::bail!("on_save_slot save path not exist: {:?}", slot);
        };
        Ok(())
    }

    fn on_save_temp(&mut self) -> anyhow::Result<()> {
        let path = Self::temp_save_path()?;
        tracing::info!("save temp path {:?}", path);
        self.save_dialogue_to_path(path)
    }

    fn on_newgame(&mut self) -> anyhow::Result<()> {
        self.with_dialogue_mut(|dialogue| dialogue.on_newgame())?;
        self.go_screen(UserScreen::Dialogue.to_string())?;
        Ok(())
    }

    fn on_load(&mut self, save_str: String) -> anyhow::Result<()> {
        self.with_dialogue_mut(|dialogue| dialogue.on_load(save_str))?;
        self.go_screen(UserScreen::Dialogue.to_string())?;
        Ok(())
    }

    fn on_continue(&mut self) -> anyhow::Result<()> {
        let path = Self::temp_save_path()?;
        let save_str = std::fs::read_to_string(path)?;
        self.with_dialogue_mut(|dialogue| dialogue.on_continue(save_str))?;
        self.go_screen(UserScreen::Dialogue.to_string())?;
        Ok(())
    }

    fn on_load_slot(&mut self, id: u8) -> anyhow::Result<()> {
        let binding = SAVE_MANAGER.with(|m| m.clone());
        let mut binding = binding.borrow_mut();
        let slot = binding.get_slot(id.into())?;
        tracing::info!("load slot path {:?}", slot.path);
        if slot.path.is_some() {
            let save_str = std::fs::read_to_string(slot.path.clone().unwrap())?;
            self.on_load(save_str)?;
        } else {
            anyhow::bail!("on_load_slot path not exist: {:?}", slot);
        };
        Ok(())
    }

    fn go_screen(&mut self, name: String) -> anyhow::Result<()> {
        if self.game_flow.borrow_mut().get_scene(&name).is_none() {
            let _ = self.game_flow.borrow_mut().ensure(name.clone())?;
            self.game_flow.borrow_mut().go_screen(&name)?;
        } else {
            self.game_flow.borrow_mut().go_screen(&name)?;
            let _ = self.game_flow.borrow_mut().ensure(name.clone())?;
        }
        // attention!: 此处为特殊处理, 一般去往主菜单时脱离游戏环境没有后退需要
        if name == UserScreen::Main.to_string() {
            self.game_flow.borrow_mut().clear_jump_path();
        }
        Ok(())
    }

    fn go_back_screen(&mut self) -> anyhow::Result<()> {
        self.game_flow
            .borrow_mut()
            .go_back_screen()
            .context("Cmd GoBack execute failed!!")?;
        Ok(())
    }

    pub fn handle_cmd(&mut self, cmd: &GameCmd) -> anyhow::Result<bool> {
        info!("{}", cmd);
        match cmd {
            GameCmd::GoScene(name) => {
                self.go_screen(name.to_string())?;
            }
            GameCmd::GoBack => {
                self.go_back_screen()?;
            }
            GameCmd::QuitGame => {
                EventSender::sender_event(GameEvent::QuitGame)?;
            }
            GameCmd::SaveTo(slot) => match slot {
                tmj_core::command::SaveSlot::Temp => {
                    self.on_save_temp()?;
                }
                tmj_core::command::SaveSlot::Slots(id) => {
                    self.on_save_slot(*id)?;
                }
            },
            GameCmd::LoadFrom(slot) => match slot {
                tmj_core::command::SaveSlot::Temp => {
                    self.on_continue()?;
                }
                tmj_core::command::SaveSlot::Slots(id) => {
                    self.on_load_slot(*id)?;
                }
            },
            GameCmd::NewGame => {
                self.on_newgame()?;
            }
            GameCmd::ContinueGame => {
                self.on_continue()?;
            }
            _ => {}
        };

        Ok(true)
    }

    pub fn draw(&self, frame: &mut Frame) {
        let screen = self.game_flow.borrow_mut().cur_screen().unwrap();
        let area = frame.area();
        let area = area.centered(
            ratatui::layout::Constraint::Length(SETTING.resolution.0),
            ratatui::layout::Constraint::Length(SETTING.resolution.1),
        );
        frame.buffer_mut().set_style(area, theme::THEME.root);

        screen.borrow_mut().draw(frame, area);
    }
}

impl EventDispatcher for Game {
    fn handle_tick(&mut self, tick: std::time::Duration) {
        AUDIOM.with_borrow_mut(|a| {
            a.update(tick);
        });
        if self.game_flow.borrow_mut().cur_screen().is_none() {
            panic!("None Sceen be set to flow!");
        }
        let screen = self.game_flow.borrow_mut().cur_screen().unwrap();
        screen.borrow_mut().handle_tick(tick);
    }

    fn on_quit(&mut self) {
        self.game_flow.borrow_mut().force_quit();
    }

    fn handle_event(&mut self, event: &GameEvent) -> Result<bool> {
        match event {
            GameEvent::CtKeyEvent(key) => self.on_key(key),
            GameEvent::CtMouseEvent(mouse) => self.on_mouse(mouse),
            GameEvent::QuitGame => self.on_quit(),
            GameEvent::ResizeTerm(w, h) => self.on_resize(*w, *h),
            _ => (),
        }

        let screen = self.game_flow.borrow().cur_screen();

        if screen.is_none() {
            panic!("None Sceen be set to flow!");
        }
        screen.unwrap().borrow_mut().handle_event(event)
    }
}
