pub mod room;
pub(crate) mod change_root;
pub(crate) mod unitselect;

use engage::gameuserdata::GameUserData;
use engage::hub::variable::HubVariable;
use engage::menu::BasicMenuResult;
use engage::menu::menu_item::BasicMenuItem;
use engage::sequence::hub::HubSequence;
use engage_il2cpp::app::IBasicMenuItemMethods;
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
pub fn sortie_top_menu_inventory_y_call(this: engage_il2cpp::app::BasicMenuItem, _method_info: unity2::OptionalMethod) -> BasicMenuResult {
    if GameUserData::get_sequence() == 2 { BasicMenuResult::se_miss() }
    else {
        UnitAssetMenuData::get().unit_select_index = 0;
        if HubSequence::get_instance().is_some() {
            if !HubVariable::get_current_scene_name().to_string().contains("Hub_Solanel") { 
                return BasicMenuResult::se_miss();
            }
        }
        let asset = UnitAssetMenuData::get();
        asset.is_shop_combat = false;
        asset.mode = MenuMode::Shop;
        asset.is_preview = true;
        CustomHubAccessoryRoom::create_bind(this.get_menu());
        BasicMenuResult::se_decide().with_close_this(true)
    }
}