use engage::gamemessage::GameMessage;
use engage::gamevariable::GameVariableManager;
use engage::mess::Mess;
use engage::titlebar::KeyHelpButton;
use engage::unit::UnitStatusField;
use unity::prelude::Il2CppString;
use crate::menu::icons::CustomMenuIcon;
use crate::{add_key_help, r_l_press, set_detail_box, CustomAssetMenu, EquipmentBoxPage, LoadResult, MenuTextCommand, ReloadPreview, UnitAssetMenuData, THUMB_DIR};
use crate::localize::MenuText;
use super::*;
#[repr(u8)]
#[derive(PartialEq, Copy, Clone)]
pub enum AssetFlag {
    EnableColor,
    EngageOutfit,
    EnableScaling,
    EnableBattleAccessories,
    EnableCrossDressing,
    RandomAppearance,
    EngagedAnimation,
    ViewMode,
    UseFaceThumbnail,
}
impl AssetFlag {
    pub fn get_rel_index(&self) -> i32 {
        match self {
            AssetFlag::EnableColor => 0,
            AssetFlag::EngageOutfit => 1,
            AssetFlag::EnableScaling => 2,
            AssetFlag::EnableBattleAccessories => 3,
            AssetFlag::EnableCrossDressing => 4,
            AssetFlag::RandomAppearance => 5,
            AssetFlag::ViewMode => 6,
            AssetFlag::EngagedAnimation => 7,
            AssetFlag::UseFaceThumbnail => 8,
        }
    }
    pub fn from_rel_index(idx: i32) -> Option<AssetFlag> {
        if idx < 8 {
            Some(
                match idx {
                    0 => AssetFlag::EnableColor,
                    1 => AssetFlag::EngageOutfit,
                    2 => AssetFlag::EnableScaling,
                    3 => AssetFlag::EnableBattleAccessories,
                    4 => AssetFlag::EnableCrossDressing,
                    5 => AssetFlag::RandomAppearance,
                    6 => AssetFlag::ViewMode,
                    7 => AssetFlag::EngagedAnimation,
                    8 => AssetFlag::UseFaceThumbnail,
                    _ => unreachable!(),
                }
            )
        }
        else { None }
    }
    pub fn is_decided(&self) -> bool {
        let mode = UnitAssetMenuData::get_flag();
        match self {
            Self::EnableColor => { mode & 1 != 0 }
            Self::RandomAppearance => { mode & 8 != 0 }
            Self::EnableBattleAccessories => { mode & 32 != 0 }
            Self::EnableScaling => { mode & 64 != 0 }
            Self::EnableCrossDressing => { mode & 128 != 0 }
            Self::EngagedAnimation => { mode & 256 != 0 }
            Self::UseFaceThumbnail => { UnitAssetMenuData::get_person_flag() & 8 != 0 }
            _ => { false }
        }
    }
}
impl CustomMenuItem for AssetFlag {
    fn get_icon(&self, _menu_item: &CustomAssetMenuItem) -> CustomMenuIcon {
        match self {
            Self::EngageOutfit|Self::EngagedAnimation => { CustomMenuIcon::EngageCommon }
            Self::EnableScaling => { CustomMenuIcon::StarBlank }
            Self::EnableColor => { CustomMenuIcon::Star }
            Self::RandomAppearance => { CustomMenuIcon::Rare }
            Self::EnableCrossDressing => { CustomMenuIcon::Body }
            Self::EnableBattleAccessories => { CustomMenuIcon::Gift }
            Self::ViewMode => { if UnitAssetMenuData::get().is_shop_combat { CustomMenuIcon::Weapon } else { CustomMenuIcon::Day } }
            Self::UseFaceThumbnail => { CustomMenuIcon::SilverCard }
        }
    }
    fn get_equipment_box_type(&self, menu_item: &CustomAssetMenuItem) -> EquipmentBoxMode {
        match self {
            Self::EnableScaling => EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::Scaling(0)),
            Self::EnableColor => EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::Color(if menu_item.index < 5 { 0 } else { 4 })),
            _ => EquipmentBoxMode::CurrentProfile,
        }
    }
    fn get_name(&self, _menu_item: &CustomAssetMenuItem) -> &'static Il2CppString {
        let mode = UnitAssetMenuData::get_flag();
        let rel = self.get_rel_index() + 20;
        match self {
            Self::EngageOutfit => {
                if mode  & 6 == 2 { format!("{}: {}", MenuTextCommand::Engage.get(), MenuTextCommand::on_off(false)) }
                else if mode  & 6 == 4 { format!("{}: {}", MenuTextCommand::Engage.get(), MenuTextCommand::Emblem.get()) }
                else { format!("{}: {}", MenuTextCommand::Engage.get(), MenuTextCommand::on_off(true)) }.into()
            }
            Self::ViewMode => {
                let base = MenuText::get_command(rel);
                let kind = if UnitAssetMenuData::get().is_shop_combat { "MID_TUT_CATEGORY_TITLE_Battle" } else { "MID_SAVEDATA_SEQ_HUB" };
                format!("{}: {}", base, Mess::get(kind)).into()
            }
            _ => { MenuText::get_command(rel) }
        }
    }
    fn get_detail_box_name(&self, _menu_item: &CustomAssetMenuItem) -> Option<&'static Il2CppString> {
        let rel = self.get_rel_index() + 20;
        match self {
            Self::EngageOutfit => Some(MenuTextCommand::Engage.get()),
            Self::EngagedAnimation => Some(format!("{} Anims", MenuTextCommand::Engage.get()).into()),
            Self::ViewMode => Some(MenuTextCommand::Mode.get()),
            _ => Some(MenuText::get_command(rel)),
        }
    }
    fn get_help(&self, _menu_item: &CustomAssetMenuItem) -> &'static Il2CppString {
        let rel = self.get_rel_index() + 20;
        let is_engaged = UnitAssetMenuData::get_current_asset_data().map(|v|{ v.set_profile[1] == UnitAssetMenuData::get_preview().selected_profile }).unwrap_or(false);
        match self {
            Self::EnableBattleAccessories => { Some("Enable some untested features.\nTurn this off if you have issues.".into()) }
            Self::EngageOutfit => {
                let mode = UnitAssetMenuData::get_flag();
                let s =
                    if mode  & 6 == 2 { MenuText::get_help(10*rel+1) }
                    else if mode  & 6 == 4 { MenuText::get_help(10*rel+2) }
                else { MenuText::get_help(10*rel) };
                if is_engaged { s } else { Some(format!("{} <color=\"red\">Profile is not set as the Engage profile.</color>", s.unwrap()).into()) }
            }
            Self::EngagedAnimation => {
                let s = MenuText::get_help(rel);
                if is_engaged { s } else { Some(format!("{}\n<color=\"red\">Profile is not set as the Engage profile.</color>", s.unwrap()).into()) }
            }
            Self::UseFaceThumbnail => {
                let s = MenuText::get_help(28);
                if let Some((keys, help)) = crate::capture::get_unit_face_keys(UnitAssetMenuData::get_unit().unwrap()).zip(s) {
                    let mut message = help.to_string();
                    if UnitAssetMenuData::get_person_flag() & 8 != 0 {
                        message += "\n";
                        message += MenuTextCommand::A.insert_right("View Files. ").to_string().as_str();
                    }
                    let key = format!("G_Face_{}", keys.0);
                    if GameVariableManager::exist(&key) {
                        let str = GameVariableManager::get_string(&key).to_string();
                        if str.contains(".png") { message += format!("File: {}. ", str).as_str(); }
                    }

                    Some(message.into())
                }
                else { s }
            }
            _ => { MenuText::get_help(rel) }
        }.unwrap()
    }
    fn get_body(&self, _menu_item: &CustomAssetMenuItem) -> &'static Il2CppString { MenuTextCommand::Settings.get() }
    fn a_call(&self, menu_item: &mut CustomAssetMenuItem) -> BasicMenuResult {
        let change_unit;
        match self {
            Self::EnableColor => {
                change_unit = true;
                UnitAssetMenuData::toggle_profile_flag(1);
            }
            Self::RandomAppearance => {
                change_unit = true;
                UnitAssetMenuData::toggle_profile_flag(8);
            }
            Self::EnableScaling => {
                change_unit = true;
                UnitAssetMenuData::toggle_profile_flag(64);
            }
            Self::EnableBattleAccessories => {
                change_unit = false;
                UnitAssetMenuData::toggle_profile_flag(32);
            }
            Self::EnableCrossDressing => {
                change_unit = false;
                UnitAssetMenuData::toggle_profile_flag(128);
            }
            Self::EngagedAnimation => {
                change_unit = false;
                UnitAssetMenuData::toggle_profile_flag(256);
            }
            Self::UseFaceThumbnail => {
                return
                if UnitAssetMenuData::get_person_flag() & 8 != 0 {
                    match UnitAssetMenuData::get().loaded_data.load_faces() {
                        LoadResult::Success => {
                            menu_item.menu.full_menu_item_list.clear();
                            FaceSelection.create_menu_items(menu_item.menu);
                            menu_item.menu.menu_kind = FaceSelection;
                            menu_item.menu.rebuild_menu();
                            add_key_help(KeyHelpButton::Minus, Mess::get("MID_MAINMENU_SAVEDATA_DELETE").to_string());
                            CustomAssetMenu::toggle_ui();
                            BasicMenuResult::se_cursor()
                        }
                        LoadResult::NoFiles => {
                            GameMessage::create_key_wait(menu_item.menu, format!("No Face thumbnails in '{}'.\nFiles are PNG of size 188x74", THUMB_DIR));
                            BasicMenuResult::se_miss()
                        }
                        LoadResult::MissingDirectory => {
                            GameMessage::create_key_wait(menu_item.menu, format!("Cannot locate directory:\n{}", THUMB_DIR));
                            BasicMenuResult::se_miss()
                        }
                    }
                }
                else { BasicMenuResult::se_miss() }
            }
            _ => { return BasicMenuResult::new(); }
        }
        menu_item.decided = self.is_decided();
        menu_item.rebuild_text();
        if change_unit { UnitAssetMenuData::reload_unit(ReloadPreview::NoScaleColor, true, None); }
        BasicMenuResult::se_cursor()
    }
    fn custom_call(&self, menu_item: &mut CustomAssetMenuItem) -> BasicMenuResult {
        let left = r_l_press(true, false, true);
        let right = r_l_press(false, true, true);
        let change_unit;
        if left || right {
            match self {
                Self::EngageOutfit => {
                    let flag = UnitAssetMenuData::get_flag();
                    if left {
                        if flag & 6 == 0 { UnitAssetMenuData::toggle_profile_flag(4); }
                        else if flag & 6 == 2 { UnitAssetMenuData::toggle_profile_flag(2); }
                        else if flag & 6 == 4 { UnitAssetMenuData::toggle_profile_flag(6);  }
                    }
                    else if right {
                        if flag & 6 == 0 { UnitAssetMenuData::toggle_profile_flag(2); }
                        else if flag & 6 == 2 { UnitAssetMenuData::toggle_profile_flag(6); }
                        else if flag & 6 == 4 { UnitAssetMenuData::toggle_profile_flag(4);  }
                    }
                    set_detail_box(None, Some(self.get_help(menu_item)), None, None);
                    change_unit = UnitAssetMenuData::get_shop_unit().is_some_and(|u| u.status.value & UnitStatusField::Engaging != 0);
                }
                Self::ViewMode => {
                    change_unit = true;
                    let v = UnitAssetMenuData::get();
                    v.is_shop_combat = !v.is_shop_combat;
                }
                Self::UseFaceThumbnail => {
                    change_unit = false;
                    UnitAssetMenuData::toggle_unit_flag(8);
                    if let Some(unit) = UnitAssetMenuData::get_unit() {
                        crate::capture::update_face(unit, UnitAssetMenuData::get_person_flag() & 8 != 0);
                    }
                    menu_item.decided = self.is_decided();
                    menu_item.rebuild_text();
                }
                _ => { return self.a_call(menu_item); }
            }
            menu_item.rebuild_text();
            if change_unit {
                UnitAssetMenuData::reload_unit(ReloadPreview::NoScaleColor, true, None);
                if *self == Self::ViewMode { UnitAssetMenuData::set_original_assets(); }
            }
            EquipmentBoxMode::CurrentProfile.update();
            BasicMenuResult::se_cursor()
        }
        else { BasicMenuResult::new() }
    }
}