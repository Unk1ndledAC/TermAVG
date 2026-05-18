use std::{cell::RefCell, fs::File, io::BufReader, path::PathBuf};

use anyhow::Result;
use strum_macros::Display;
use tmj_core::{
    audio::{AudioManager, AudioSource},
    pathes,
    script::script_sym,
};

#[derive(Clone, Hash, PartialEq, Debug, Display)]
pub enum Tracks {
    Bgm,
    Voice,
    /// 环境音效（与 BGM、语音独立，脚本 `env_effect` 使用）
    EnvEffect,
    MainMenuBgm,
    Effect,
    Effect1,
    Effect2,
}

impl Eq for Tracks {}

pub fn load_audio(file: impl ToString) -> Result<AudioSource> {
    let path = pathes::path(file.to_string());
    let file = File::open(path)?;
    let source = rodio::Decoder::new(BufReader::new(file))?;
    Ok(Box::new(source))
}

pub fn load_audio_from_abspath(path: &PathBuf) -> Result<AudioSource> {
    let file = File::open(path)?;
    let source = rodio::Decoder::new(BufReader::new(file))?;
    Ok(Box::new(source))
}

thread_local! {
    pub static AUDIOM: RefCell<AudioManager<Tracks>> = RefCell::new(AudioManager::new().unwrap());
}

script_sym!(FADE_IN, Member, "BGM 淡入模式标识");
script_sym!(FADE_OUT, Member, "BGM 淡出模式标识");
script_sym!(TRANSITION, Member, "BGM 交叉过渡模式标识");
