use engage::unit::Gender;
use engage::gamemessage::GameMessage;
use engage::mess::Mess;
use crate::{LoadResult, UnitAssetMenuData};
use crate::localize::MenuText;
use super::*;
#[derive(PartialEq, Copy, Clone)]
pub enum AssetDataMode {
    Import,
    Export,
    ExportPreview,
}

impl CustomMenuItem for AssetDataMode {
    fn get_icon(&self, _menu_item: &CustomAssetMenuItem) -> CustomMenuIcon { CustomMenuIcon::Satchel }
    fn get_equipment_box_type(&self, _menu_item: &CustomAssetMenuItem) -> EquipmentBoxMode { EquipmentBoxMode::CurrentProfile }
    fn get_name(&self, _menu_item: &CustomAssetMenuItem) -> &'static Il2CppString {
        self.get_detail_box_name(_menu_item).unwrap()
    }
    fn get_detail_box_name(&self, _menu_item: &CustomAssetMenuItem) -> Option<&'static Il2CppString> {
        match self {
            Self::Import => Some(Mess::get("MID_SAVEDATA_LOAD_YES")),
            Self::Export => Some(Mess::get("MID_SAVEDATA_SAVE_TITLE")),
            Self::ExportPreview => Some(format!("{} [Preview]", Mess::get("MID_SAVEDATA_SAVE_TITLE")).into())
        }
    }
    fn get_help(&self, _menu_item: &CustomAssetMenuItem) -> &'static Il2CppString {
        match self {
            Self::Export => MenuText::get_help(5).unwrap(),
            Self::Import => MenuText::get_help(6).unwrap(),
            Self::ExportPreview => MenuText::get_help(9).unwrap(),
        }
    }
    fn get_body(&self, _menu_item: &CustomAssetMenuItem) -> &'static Il2CppString { "Data".into() }
    fn a_call(&self, menu_item: &mut CustomAssetMenuItem) -> BasicMenuResult {
        match self {
            Self::Import => {
                let gender =
                    if UnitAssetMenuData::get_flag() & 128 != 0 { Gender::None }
                    else if UnitAssetMenuData::get_current_dress_gender() == 2 { Gender::Female }
                    else { Gender::Male };
                match UnitAssetMenuData::get().loaded_data.load_files(gender){
                    LoadResult::Success => {
                        menu_item.menu.kind = 1;
                        menu_item.menu.full_menu_item_list.clear();
                        menu_item.menu.save_current_select();
                        menu_item.menu.menu_kind = LoadData;
                        LoadData.create_menu_items(menu_item.menu);
                        menu_item.menu.rebuild_menu();
                        BasicMenuResult::se_decide()
                    }
                    LoadResult::NoFiles => {
                        GameMessage::create_key_wait(menu_item.menu, format!("No valid data found in\n{}", crate::INPUT_DIR).as_str());
                        BasicMenuResult::se_miss()
                    }
                    LoadResult::MissingDirectory => {
                        GameMessage::create_key_wait(menu_item.menu, format!("Missing input directory:\n{}", crate::INPUT_DIR).as_str());
                        BasicMenuResult::se_miss()
                    }
                }
            }
            Self::ExportPreview => { output(menu_item, true) }
            Self::Export => { output(menu_item, false) }
        }
    }
}
fn output(menu_item: &mut CustomAssetMenuItem, preview: bool) -> BasicMenuResult {
    let (filename, data, saved) = crate::output_unit_result(preview);
    if saved {
        GameMessage::create_key_wait(menu_item.menu, format!("Saved\nResult: {}\nData: {}",filename, data).as_str());
        BasicMenuResult::se_cursor()
    }
    else {
        GameMessage::create_key_wait(menu_item.menu, "Failed to export data/result.") ;
        BasicMenuResult::se_miss()
    }
}

