use std::{
    cell::RefCell,
    collections::HashMap,
    fs,
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tmj_core::{pathes, script::ScriptValue};

use crate::audio::{AUDIOM, Tracks};

pub const BGM_VOLUME: &str = "bgm_volume";
pub const ENV_VOLUME: &str = "env_volume";
pub const EFFECT_VOLUME: &str = "effect_volume";

#[derive(Serialize, Deserialize)]
#[serde(default)]
struct GameSettingFile {
    bgm_volume: f64,
    env_volume: f64,
    effect_volume: f64,
}

impl Default for GameSettingFile {
    fn default() -> Self {
        Self {
            bgm_volume: 1.0,
            env_volume: 1.0,
            effect_volume: 1.0,
        }
    }
}

pub struct GameSetting {
    pub fields: HashMap<String, ScriptValue>,
}

impl Default for GameSetting {
    fn default() -> Self {
        let mut fields = HashMap::new();
        fields.insert(BGM_VOLUME.to_string(), ScriptValue::Float(1.0));
        fields.insert(ENV_VOLUME.to_string(), ScriptValue::Float(1.0));
        fields.insert(EFFECT_VOLUME.to_string(), ScriptValue::Float(1.0));
        Self { fields }
    }
}

impl From<GameSettingFile> for GameSetting {
    fn from(value: GameSettingFile) -> Self {
        let mut fields = HashMap::new();
        fields.insert(BGM_VOLUME.to_string(), ScriptValue::Float(value.bgm_volume));
        fields.insert(ENV_VOLUME.to_string(), ScriptValue::Float(value.env_volume));
        fields.insert(EFFECT_VOLUME.to_string(), ScriptValue::Float(value.effect_volume));
        Self { fields }
    }
}

impl From<&GameSetting> for GameSettingFile {
    fn from(value: &GameSetting) -> Self {
        Self {
            bgm_volume: value
                .fields
                .get(BGM_VOLUME)
                .and_then(ScriptValue::to_number)
                .unwrap_or(1.0),
            env_volume: value
                .fields
                .get(ENV_VOLUME)
                .and_then(ScriptValue::to_number)
                .unwrap_or(1.0),
            effect_volume: value
                .fields
                .get(EFFECT_VOLUME)
                .and_then(ScriptValue::to_number)
                .unwrap_or(1.0),
        }
    }
}

impl GameSetting {
    fn setting_file_path() -> std::path::PathBuf {
        pathes::path("game_setting.toml")
    }

    fn persist(&self) -> Result<()> {
        let path = Self::setting_file_path();
        let content = toml::to_string(&GameSettingFile::from(self))?;
        fs::write(path, content)?;
        Ok(())
    }

    fn apply_known_field(var_name: &str, value: &ScriptValue) -> Result<()> {
        let volume = value
            .to_number()
            .context(format!("{var_name} must be number"))?
            .clamp(0.0, 1.0) as f32;

        AUDIOM.with_borrow_mut(|a| match var_name {
            BGM_VOLUME => {
                if let Some(track) = a.track_mut(&Tracks::Bgm) {
                    track.set_volume_multiplier(volume);
                }
            }
            ENV_VOLUME => {
                if let Some(track) = a.track_mut(&Tracks::EnvEffect) {
                    track.set_volume_multiplier(volume);
                }
            }
            EFFECT_VOLUME => {
                if let Some(track) = a.track_mut(&Tracks::Effect) {
                    track.set_volume_multiplier(volume);
                }
                if let Some(track) = a.track_mut(&Tracks::Effect1) {
                    track.set_volume_multiplier(volume);
                }
                if let Some(track) = a.track_mut(&Tracks::Effect2) {
                    track.set_volume_multiplier(volume);
                }
            }
            _ => {}
        });
        Ok(())
    }

    pub fn apply_setting(&self) -> Result<()> {
        for (key, value) in &self.fields {
            Self::apply_known_field(key.as_str(), value)?;
        }
        Ok(())
    }

    pub fn apply_field(&mut self, var_name: String, value: ScriptValue) -> Result<()> {
        self.fields.insert(var_name.clone(), value.clone());
        Self::apply_known_field(&var_name, &value)?;
        Ok(())
    }

    pub fn get_number(&self, var_name: &str) -> Option<f64> {
        self.fields.get(var_name).and_then(ScriptValue::to_number)
    }

    pub fn persist_to_file(&self) -> Result<()> {
        self.persist()
    }
}

fn read_or_create_setting() -> Result<GameSetting> {
    let path = GameSetting::setting_file_path();
    if fs::exists(&path)? {
        let content = fs::read_to_string(&path).context("game_setting.toml unreadable")?;
        let file_setting = toml::from_str::<GameSettingFile>(&content)?;
        Ok(file_setting.into())
    } else {
        let setting = GameSetting::default();
        setting.persist()?;
        Ok(setting)
    }
}

thread_local! {
    pub static GAME_SETTING: RefCell<GameSetting> = RefCell::new(
        read_or_create_setting().unwrap_or_else(|e| {
            tracing::error!("load game_setting.toml failed: {:?}", e);
            GameSetting::default()
        })
    );
}
