use engage::mess::Mess;
use unity::prelude::Il2CppString;
use crate::menu::icons::CustomMenuIcon;
use crate::{MenuText, MenuTextCommand, UnitAssetMenuData};

#[derive(PartialEq, Copy, Clone)]
pub enum Profile {
    Battle,
    EngagedDark,
    Hub,
    Alt1,
    Alt2,
}
impl Profile {
    pub fn get_icon(&self) -> CustomMenuIcon {
        match self {
            Profile::Battle => CustomMenuIcon::Weapon,
            Profile::EngagedDark => CustomMenuIcon::EngageCommon,
            Profile::Hub=> CustomMenuIcon::Day,
            _ => CustomMenuIcon::KeyItem,
        }
    }
    pub fn to_index(&self) -> usize {
        match self {
            Self::Battle => 0,
            Self::EngagedDark => 1,
            Self::Hub => 2,
            Self::Alt1 => 3,
            Self::Alt2 => 4,
        }
    }
    pub fn left(&self) -> Self {
        if UnitAssetMenuData::get().god_mode {
            match self {
                Self::Battle => Self::Hub,
                Self::EngagedDark => Self::Battle,
                Self::Hub => Self::EngagedDark,
                _ => Self::Battle,
            }
        }
        else {
            match self {
                Self::Battle => Self::Alt2,
                Self::EngagedDark => Self::Battle,
                Self::Hub => Self::EngagedDark,
                Self::Alt1 => Self::Hub,
                Self::Alt2 => Self::Alt1,
            }
        }
    }
    pub fn right(&self) -> Self {
        if UnitAssetMenuData::get().god_mode {
            match self {
                Self::Battle => Self::EngagedDark,
                Self::EngagedDark => Self::Hub,
                Self::Hub => Self::Battle,
                _ => Self::Battle,
            }
        }
        else {
            match self {
                Self::Battle => Self::EngagedDark,
                Self::EngagedDark => Self::Hub,
                Self::Hub => Self::Alt1,
                Self::Alt1 => Self::Alt2,
                Self::Alt2 => Self::Battle,
            }
        }
    }
    pub fn from_index(index: i32) -> Self {
        match index{
            0 => Self::Battle,
            1 => Self::EngagedDark,
            2 => Self::Hub,
            3 => Self::Alt1,
            4 => Self::Alt2,
            _ => unreachable!(),
        }
    }
    pub fn get_name(&self) -> &'static Il2CppString {
        match self {
            Self::Battle=> Mess::get("MID_TUT_CATEGORY_TITLE_Battle"),
            Self::EngagedDark => {
                if UnitAssetMenuData::get().god_mode { Mess::get("MCID_M007") }
                else { MenuTextCommand::Engage.get() }
            },
            Self::Hub => Mess::get("MID_SAVEDATA_SEQ_HUB"),
            Self::Alt1 => format!("{} 1", MenuText::get_command(40)).into(),
            Self::Alt2 => format!("{} 2", MenuText::get_command(40)).into(),
        }
    }
}