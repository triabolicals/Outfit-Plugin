use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::sync::{OnceLock, RwLock};
use engage::language::{Language, LanguageLangs};
use engage::mess::Mess;
use unity::prelude::Il2CppString;

pub static LOCAL_TEXT: OnceLock<RwLock<MenuText>> = OnceLock::new();

pub const KEY: [&str; 14] = ["A", "B", "X", "Y", "L", "R", "ZL", "ZR", "Plus", "Minus", "Up", "Down", "Right", "Left"];
pub const MIDS: [&str; 38] = [
    "MID_CONFIG_ROD_DANCE_ON", "MID_CONFIG_ROD_DANCE_OFF", "MID_CONFIG_GAMESPEED_NOMAL", "MID_MATCH_Random",
    "MID_SORTIE_SKILL_CATEGORY_PERSON", "MID_SYS_Class", "MID_SORTIE_SKILL_CATEGORY_GOD", "MID_SORTIE_SKILL_CATEGORY_ENGAGE",
    "MID_MENU_UNIT_LIST", "MID_MENU_REFINE_SHOP_REFINE_GODSYMBOL", "MID_KEYHELP_MENU_UNIT_SELECT", "MID_SYS_Me", "MID_SYS_SP",
    "MID_GAMESTART_PLAYER_GENDER_SELECT_TITLE", "MID_Hub_amiibo_Accessory_Trade", "MID_ITEMMENU_ITEM_ALL",  "MTID_Engage", "MID_SYS_Grow_Fixed",
    "MID_SYS_Skill", "MID_GAMESTART_GROWMODE_SELECT_TITLE", "MID_H_INFO_Param_Correction_God", "MID_MENU_ACCESSORY_SHOP_PART_BODY", "MID_MAINMENU_LANGUAGE_VOICE",
    "MID_SAVEDATA_SEQ_HUB", "MID_TUT_CATEGORY_TITLE_Battle", "MID_MATCH_Map_UpRoad_No", "MID_Decision", "MID_SAVEDATA_LOAD_YES", "MID_SAVEDATA_SAVE_TITLE",
    "MID_ProfileCard_Card_GameMode", "MID_MENU_CONFIG", "MID_KEYHELP_MENU_SELECT", "MID_MENU_RESET", "MID_ProfileCard_Stamp_Unit", "MID_ProfileCard_Stamp_Weapon",
    "MID_ProfileCard_Stamp_GodWeapon", "MID_CARD_KEYHELP_EDIT", "MID_MAINMENU_SAVEDATA_COPY"
];
const ADDED: [&str; 10] = [
    "Original", "Reset", "Custom", "Alt", "Male", "Female", "Generic", "Data", "Preset", "Player"
];

#[repr(i32)]
#[derive(Clone, Copy)]
pub enum MenuTextCommand {
    On = 0,
    Off = 1,
    Normal = 2,
    Random = 3,
    Personal = 4,
    Class = 5,
    Sync = 6,
    Engage = 7,
    List = 8,
    Emblems = 9,
    View = 10,
    SelfText = 11,
    SP = 12,
    Appearance = 13,
    Outfits = 14,
    ALL = 15,
    EmblemEnergy = 16,
    Fixed = 17,
    Skill = 18,
    GrowMode = 19,
    Emblem = 20,
    Body = 21,
    Voice = 22,
    Somniel = 23,
    Battle = 24,
    Cancel = 25,
    Confirm = 26,
    Load = 27,
    Save = 28,
    Mode = 29,
    Settings = 30,
    Select = 31,
    Quit = 32,
    Units = 33,
    Weapons = 34,
    EngageWeapons = 35,
    Edit = 36,
    Copy = 37,
    Original = 50,
    Reset = 51,
    Custom = 52,
    Alt = 53,
    Male = 54,
    Female = 55,
    Generic = 56,
    Data = 57,
    Preset = 58,
    Player = 59,
    A = 200,
    B = 201,
    X = 202,
    Y = 203,
    L = 204,
    R = 205,
    ZL = 206,
    ZR = 207,
    Plus = 208,
    Minus = 209,
    Up = 210,
    Down = 211,
    Left = 212,
    Right = 213,
    LeftRight = 214,
}
impl Display for MenuTextCommand {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get())
    }
}

impl MenuTextCommand {
    pub fn on_off(on: bool) -> &'static Il2CppString { if on { Self::On } else { Self::Off }.get() }
    pub fn to_right(self, other: Self) -> &'static Il2CppString {
        format!("{}{}", self.get(), other.get()).into()
    }
    pub fn insert_right<T: Display>(self, str: T) -> &'static Il2CppString {
        format!("{} {}", self.get(), str).into()
    }
    pub fn insert_left<T: Display>(self, str: T) -> &'static Il2CppString {
        format!("{} {}", str, self.get()).into()
    }
    pub fn get_with_sys_sprite(self, sys: &str) -> &'static Il2CppString {
        format!("{}{}", Mess::create_sprite_tag_str(2, sys), self.get()).into()
    }
    pub fn get_with_value<T: Display>(self, value: T) -> &'static Il2CppString {
        format!("{}: {}", self.get(), value).into()
    }
    pub fn get(&self) -> &'static Il2CppString {
        let index = *self as usize;
        if index < MIDS.len() { Mess::get(MIDS[index]) }
        else if index >= 50 && index < 50 + ADDED.len() { ADDED[index - 50].into() }
        else if index >= 200 && index < 214 {
            Mess::create_sprite_tag_str(2, KEY[index-200])
        }
        else if index  == 214 {
            format!("{}{}",
                    Mess::create_sprite_tag_str(2, "Left"),
                    Mess::create_sprite_tag_str(2, "Right")
            ).into()
        }
        else { format!("C{}", index).into() }
    }
    pub fn get_from_index(index: i32) -> &'static mut Il2CppString {
        if index < MIDS.len() as i32 { Mess::get(MIDS[index as usize]) }
        else { format!("C {}", index).into() }
    }
    pub fn get_gender(is_female: bool) -> Self {
        if is_female { Self::Female } else { Self::Male }
    }
}

pub struct MenuText {
    pub help: HashMap<i32, &'static str>,
    pub command: HashMap<i32, &'static str>,
}
impl MenuText {
    pub fn get_help(id: i32) -> Option<&'static Il2CppString> {
        let texts = LOCAL_TEXT.get_or_init(|| RwLock::new(Self::init())).read().ok()?;
        texts.help.get(&id)
            .map(|s| {
                let mut str = s.replace("\\n", "\n");
                KEY.iter().for_each(|&k| {
                    if str.contains(format!("$({})", k).as_str()) {
                        str = str.replace(format!("$({})", k).as_str(), Mess::create_sprite_tag_str(2, k).to_string().as_str());
                    }
                });
                str.into()
            })
    }
    pub fn init() -> Self {
        let help = Self::parse_to_map(Self::get_help_text());
        let command = Self::parse_to_map(Self::get_command_text());
        Self { help, command }
    }
    pub fn get_command(id: i32) -> &'static Il2CppString {
        if let Some(texts) = LOCAL_TEXT.get_or_init(|| RwLock::new(Self::init())).read().ok(){
            let alt = (id / 10) * 10;
            if let Some(c) = texts.command.get(&id){
                return c.into();
            }
            else if let Some(c) = texts.command.get(&alt){
                return c.into();
            }
        }
        format!("C-{}", id).into()
    }
    pub fn parse_to_map(content: &'static str) -> HashMap<i32, &'static str> {
        content.lines().flat_map(|l| {
            l.split_once("\t").or_else(|| l.split_once(' '))
                .and_then(|x| x.0.parse::<i32>().ok().zip(Some(x.1)))
        }).collect()
    }
    fn get_help_text() -> &'static str {
        match Language::get_lang() {
            LanguageLangs::CNTraditional => include_str!("localize/help/tw.txt"),
            _ => include_str!("localize/help/en.txt"),
        }
    }
    fn get_command_text() -> &'static str {
        match Language::get_lang() {
            LanguageLangs::CNTraditional => include_str!("localize/command/tw.txt"),
            /*
            LanguageLangs::JPJapanese => include_str!("localize/command/ja.txt"),
            LanguageLangs::USFrench|LanguageLangs::EUFrench => include_str!("localize/command/fr.txt"),
            LanguageLangs::USSpanish|LanguageLangs::EUSpanish => include_str!("localize/command/es.txt"),
            LanguageLangs::EUItalian => include_str!("localize/command/it.txt"),

            LanguageLangs::CNSimplified =>
            LanguageLangs::KRKorean => {}
            */
            _ => { include_str!("localize/command/en.txt") }
        }
    }
}