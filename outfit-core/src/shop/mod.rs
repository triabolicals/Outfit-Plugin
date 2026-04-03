pub mod room;
pub(crate) mod change_root;
pub(crate) mod unitselect;

use engage::gamedata::Gamedata;
use engage::gamedata::skill::SkillDataCategorys;
use engage::gameuserdata::GameUserData;
use engage::menu::BasicMenuResult;
use engage::menu::menu_item::BasicMenuItem;
use unity::macro_context::Il2CppClass;
use unity::prelude::OptionalMethod;
use crate::data::room::CustomHubAccessoryRoom;
use crate::{MenuMode, UnitAssetMenuData};

pub fn sortie_menu_x_call_edit() {
    if let Some(k) =
        Il2CppClass::from_name("App", "SortieTopMenu").ok()
            .and_then(|k| k.get_nested_types().iter().find(|c| c.get_name() == "InventoryMenuItem".to_string()))
            .and_then(|s| Il2CppClass::from_il2cpptype(s.get_type()).ok())
            .and_then(|s| s.get_virtual_method_mut("XCall"))
    {
        k.method_ptr = sortie_top_menu_inventory_y_call as _;
    }
    if let Some(k) =Il2CppClass::from_name("App", "HubMenu").ok()
        .and_then(|k| k.get_nested_types().iter().find(|c| c.get_name() == "InventoryItem".to_string()))
        .and_then(|s| Il2CppClass::from_il2cpptype(s.get_type()).ok())
    {
        k.get_virtual_method_mut("XCall").map(|x| x.method_ptr = sortie_top_menu_inventory_y_call as _);
        k.get_virtual_method_mut("PlusCall").map(|x| x.method_ptr = debug_plus_call as _);
    }
    if let Some(k) = Il2CppClass::from_name("App", "GmapMenuSequence").ok()
        .and_then(|k| k.get_nested_types().iter().find(|c| c.get_name() == "GmapMenu".to_string()))
        .and_then(|k| k.get_nested_types().iter().find(|c| c.get_name() == "InventoryItem".to_string()))
        .and_then(|s| Il2CppClass::from_il2cpptype(s.get_type()).ok())
        .and_then(|s| s.get_virtual_method_mut("XCall"))
    {
        k.method_ptr = sortie_top_menu_inventory_y_call as _;
    }
}
pub fn sortie_top_menu_inventory_y_call(this: &mut BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    if GameUserData::get_sequence() == 2 { BasicMenuResult::se_miss() }
    else {
        let asset = UnitAssetMenuData::get();
        asset.is_shop_combat = GameUserData::get_sequence() != 4;
        asset.mode = MenuMode::Shop;
        asset.is_preview = true;
        CustomHubAccessoryRoom::create_bind(this.menu);
        BasicMenuResult::se_decide().with_close_this(true)
    }
}
pub fn debug_plus_call(_this: &mut BasicMenuItem, _method_info: OptionalMethod) -> BasicMenuResult {
    if let Some(skill) = engage::gamedata::skill::SkillData::try_index_get_mut(0) {
        for x in 1..10 { skill.weapon_level.levels[x] = 5; }
        skill.weapon_level_mask.value = 1023;
        skill.sync_skills.add_sid("SID_竜石装備",SkillDataCategorys::None, 0);
        skill.sync_skills.add_sid("SID_弾丸装備",SkillDataCategorys::None, 0);
    }
    BasicMenuResult::se_decide()
}