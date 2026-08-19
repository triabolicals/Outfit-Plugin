use engage::app::IBasicMenuItemMethods;
use crate::{LoadResult, UnitAssetMenuData, localize::MenuText};
use super::*;

#[derive(PartialEq, Copy, Clone)]
pub enum AssetDataMode {
    Import,
    Export,
    ExportPreview,
}

impl CustomMenuItem for AssetDataMode {
    fn get_icon(&self, _: CustomAssetMenuItem3) -> CustomMenuIcon { CustomMenuIcon::Satchel }
    fn get_equipment_box_type(&self, _: CustomAssetMenuItem3) -> EquipmentBoxMode { EquipmentBoxMode::CurrentProfile }
    fn get_name(&self, _menu_item: CustomAssetMenuItem3) -> unity::Il2CppString {
        self.get_detail_box_name(_menu_item).unwrap()
    }
    fn get_detail_box_name(&self, _menu_item: CustomAssetMenuItem3) -> Option<unity::Il2CppString> {
        match self {
            Self::Import => Some(engage::app::Mess::get("MID_SAVEDATA_LOAD_YES")),
            Self::Export => Some(engage::app::Mess::get("MID_SAVEDATA_SAVE_TITLE")),
            Self::ExportPreview => Some(format!("{} [Preview]", engage::app::Mess::get("MID_SAVEDATA_SAVE_TITLE")).into())
        }
    }
    fn get_help(&self, _menu_item: CustomAssetMenuItem3) -> unity::Il2CppString {
        match self {
            Self::Export => MenuText::get_help(5).unwrap(),
            Self::Import => MenuText::get_help(6).unwrap(),
            Self::ExportPreview => MenuText::get_help(9).unwrap(),
        }
    }
    fn get_body(&self, _menu_item: CustomAssetMenuItem3) -> unity::Il2CppString { "Data".into() }
    fn a_call(&self, menu_item: CustomAssetMenuItem3) -> BasicMenu_Result {
        match self {
            Self::Import => {
                let gender =
                    if UnitAssetMenuData::get_flag() & 128 != 0 { engage::app::Gender::none() }
                    else if UnitAssetMenuData::get_current_dress_gender() == 2 { engage::app::Gender::female() }
                    else { engage::app::Gender::male() };
                match UnitAssetMenuData::get().loaded_data.load_files(gender){
                    LoadResult::Success => {
                        let menu = menu_item.get_asset_menu();
                        menu.rebuild_menu(LoadData, true);
                        BasicMenu_Result::se_decide()
                    }
                    LoadResult::NoFiles => {
                        engage::app::GameMessage::create_key_wait(menu_item.get_menu(), format!("No valid data found in\n{}", crate::INPUT_DIR));
                        BasicMenu_Result::se_miss()
                    }
                    LoadResult::MissingDirectory => {
                        engage::app::GameMessage::create_key_wait(menu_item.get_menu(), format!("Missing input directory:\n{}", crate::INPUT_DIR).as_str());
                        BasicMenu_Result::se_miss()
                    }
                }
            }
            Self::ExportPreview => { output(menu_item, true) }
            Self::Export => { output(menu_item, false) }
        }
    }
}
fn output(menu_item: CustomAssetMenuItem3, preview: bool) -> BasicMenu_Result {
    let (filename, data, saved) = crate::output_unit_result(preview);
    if saved {
        engage::app::GameMessage::create_key_wait(menu_item.get_menu(), format!("Saved\nResult: {}\nData: {}",filename, data).as_str());
        BasicMenu_Result::se_cursor()
    }
    else {
        engage::app::GameMessage::create_key_wait(menu_item.get_menu(),"Failed to export data/result.") ;
        BasicMenu_Result::se_miss()
    }
}

