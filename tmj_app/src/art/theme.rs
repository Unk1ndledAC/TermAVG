
use ratatui::style::{Color, Style};

pub struct Theme {
    pub root: Style,
    pub content: Style,
    pub borders: Style,
    pub key_binding: KeyBinding,
    pub dialouge: Dialogue,
    pub history: History,
    pub main_menu: MainMenu,
    pub slot_list: SlotList,
    pub save_screen: SaveScreenTheme,
    pub load_screen: LoadScreenTheme,
}

pub struct KeyBinding {
    pub key: Style,
    pub description: Style,
}

pub struct Dialogue {
    pub inbox: Style,
    pub block: Style,
    pub black_edge: Style,
    pub name: Style,
    pub charpter_subtitle: Style,
    pub charpter_title: Style,
    pub background: Style,
}

pub struct History {
    pub base: Style,
    pub item_border: Style,
    pub say_item: Style,
    pub text_item: Style,
}

pub struct MainMenu {
    pub block: Style,
    pub item: Style,
    pub disabled_item: Style,
    pub selected_item: Style,
}

pub struct SlotList {
    pub save: SlotListVariant,
    pub load: SlotListVariant,
    pub selected_item: Style,
}

pub struct SlotListVariant {
    pub slot_id: Style,
    pub slot_info: Style,
    pub empty_item: Style,
    pub title: Style,
    pub hint: Style,
}

pub struct SaveScreenTheme {
    pub rename_block: Style,
    pub rename_text: Style,
}

pub struct LoadScreenTheme {
    pub confirm_block: Style,
    pub confirm_yes: Style,
    pub confirm_no: Style,
}

pub const THEME: Theme = Theme {
    root: Style::new().bg(BLACK),
    content: Style::new().bg(DARK_BLUE).fg(LIGHT_GRAY),
    borders: Style::new().fg(LIGHT_GRAY),
    key_binding: KeyBinding {
        key: Style::new().fg(BLACK).bg(MID_GRAY),
        description: Style::new().fg(MID_GRAY).bg(BLACK),
    },
    dialouge: Dialogue {
        name: Style::new().fg(WHITE).bg(BLACK),
        inbox: Style::new().bg(DARK_BLUE).fg(WHITE),
        block: Style::new().bg(DARK_BLUE).fg(LIGHT_GRAY),
        black_edge: Style::new().bg(BLACK).fg(WHITE),
        charpter_subtitle: Style::new().fg(LIGHT_GRAY),
        charpter_title: Style::new().fg(WHITE),
        background: Style::new().bg(BLACK),
    },
    history: History{
        base: Style::new().bg(DARK_BLUE),
        item_border: Style::new().fg(LIGHT_GRAY),
        say_item: Style::new().fg(WHITE),
        text_item: Style::new().fg(LIGHT_GRAY)
    },
    main_menu: MainMenu {
        block: Style::new().bg(DARK_BLUE),
        item: Style::new().fg(WHITE),
        disabled_item: Style::new().fg(DARK_GRAY),
        selected_item: Style::new().fg(LIGHT_YELLOW),
    },
    slot_list: SlotList {
        save: SlotListVariant {
            slot_id: Style::new().fg(LTY_BLUE),
            slot_info: Style::new().fg(LTY_BLUE),
            empty_item: Style::new().fg(DARK_LTY_BLUE),
            title: Style::new().fg(LTY_BLUE),
            hint: Style::new().fg(DARK_LTY_BLUE),
        },
        load: SlotListVariant {
            slot_id: Style::new().fg(ORANGE),
            slot_info: Style::new().fg(ORANGE),
            empty_item: Style::new().fg(DARK_ORANGE),
            title: Style::new().fg(ORANGE),
            hint: Style::new().fg(DARK_ORANGE),
        },
        selected_item: Style::new().bg(WHITE).fg(BLACK),
    },
    save_screen: SaveScreenTheme {
        rename_block: Style::new().fg(LTY_BLUE),
        rename_text: Style::new().fg(WHITE),
    },
    load_screen: LoadScreenTheme {
        confirm_block: Style::new().fg(ORANGE),
        confirm_yes: Style::new().fg(LIGHT_GREEN),
        confirm_no: Style::new().fg(DARK_ORANGE),
    },
};

pub const DARK_BLUE: Color = Color::Rgb(16, 24, 48);
pub const LIGHT_BLUE: Color = Color::Rgb(64, 96, 192);
pub const LIGHT_YELLOW: Color = Color::Rgb(192, 192, 96);
pub const LIGHT_GREEN: Color = Color::Rgb(64, 192, 96);
pub const LIGHT_RED: Color = Color::Rgb(192, 96, 96);
pub const ORANGE: Color = Color::Rgb(255, 165, 64);
pub const DARK_ORANGE: Color = Color::Rgb(166, 102, 36);
pub const DARK_LTY_BLUE: Color = Color::Rgb(70, 120, 150);
pub const BLACK: Color = Color::Rgb(8, 8, 8); // not really black, often #080808
pub const DARK_GRAY: Color = Color::Rgb(68, 68, 68);
pub const MID_GRAY: Color = Color::Rgb(128, 128, 128);
pub const LIGHT_GRAY: Color = Color::Rgb(188, 188, 188);
pub const WHITE: Color = Color::Rgb(238, 238, 238); // not really white, often #eeeeee
pub const LTY_BLUE: Color = Color::from_u32(0x66ccff_u32);
