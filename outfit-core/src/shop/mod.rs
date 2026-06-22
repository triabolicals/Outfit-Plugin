pub mod room;
pub(crate) mod change_root;
pub(crate) mod unitselect;

use engage::app::{BasicMenu_Result, IBasicMenuItemMethods, IGameUserDataMethods, ISingletonClass_1Methods};
use crate::data::room::CustomHubAccessoryRoom;
use crate::{get_virtual_methods_mut, MenuMode, UnitAssetMenuData};

pub fn sortie_menu_x_call_edit() {
    get_virtual_methods_mut("App", "SortieTopMenu.InventoryMenuItem", "XCall").map(|k| k.method_ptr = sortie_top_menu_inventory_y_call as _);
    get_virtual_methods_mut("App", "HubMenu.InventoryItem", "XCall").map(|x| x.method_ptr = sortie_top_menu_inventory_y_call as _);
    get_virtual_methods_mut("App", "GmapMenuSequence.GmapMenu.InventoryItem", "XCall").map(|x| x.method_ptr = sortie_top_menu_inventory_y_call as _);
}
pub fn sortie_top_menu_inventory_y_call(this: engage::app::BasicMenuItem, _method_info: unity::OptionalMethod) -> BasicMenu_Result {
    if engage::app::GameUserData::get_instance().get_sequence().value == 2 { BasicMenu_Result::se_miss() }
    else {
        UnitAssetMenuData::get().unit_select_index = 0;
        let asset = UnitAssetMenuData::get();
        asset.is_shop_combat = false;
        asset.mode = MenuMode::Shop;
        asset.is_preview = true;
        CustomHubAccessoryRoom::create_bind(this.get_menu());
        BasicMenu_Result::close_decide()
    }
}