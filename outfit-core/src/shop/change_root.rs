use engage::gamesound::GameSound;
use engage::menu::menus::accessory::change::{AccessoryShopChangeRoot, AccessoryShopChangeRootProc};
use unity::prelude::*;
use crate::data::room::hub_room_set_by_result;
pub use crate::menu::*;
use crate::room::ReloadType;

pub fn edit_accessory_root_change_unit(change_root: &mut AccessoryShopChangeRootProc) {
    if let Some(change_next) = change_root.change_unit_next.as_mut() {
        change_next.method_ptr = accessory_change_root_next_unit as _;
    }
    if let Some(change_previous) = change_root.change_unit_previous.as_mut() {
        change_previous.method_ptr = accessory_change_root_previous_unit as _;
    }
    if let Some(start_watching) = change_root.start_watching.as_mut() {
        start_watching.method_ptr = accessory_change_root_key_on_start_watching as _;
    }
    if let Some(end_watching) = change_root.end_watching.as_mut() {
        end_watching.method_ptr = accessory_change_root_key_end_watching as _;
    }
    if let Some(on_show_ui) = change_root.show_ui.as_mut() { on_show_ui.method_ptr = accessory_change_root_key_on_show_ui as _; }
}
fn change_character(this: &'static mut AccessoryShopChangeRoot, next: bool, watching: bool) {
    UnitAssetMenuData::commit();
    let data = UnitAssetMenuData::get();
    data.unit_select.change(next);
    if let Some(selected) = data.unit_select.get_selected() {
        UnitAssetMenuData::set_by_hash(selected.hash);
        let result = selected.get_result(data.is_hub);
        CustomAssetMenu::init(this.change_menu, false);
        hub_room_set_by_result(Some(result), ReloadType::All);
        if let Some(name) = selected.get_name() { this.unit_name.set_text(name, true); }
        GameSound::post_event("Chara_Change", None);
    }
    if watching {
        this.menu_object.set_active(false);
        this.detail_info_window.set_active(false);
        this.change_menu.status.value |= 4;
        if let Some(object) = GameObject::find("KeyHelpCamera") {
            if let Some(transform) = object.get_component_by_type::<RectTransform>() {
                let mut pos = transform.get_position();
                pos.y = 580.0;
                transform.set_position(pos);
            }
        }
    }
    else { this.detail_info.show(); }
}
pub extern "C" fn accessory_change_root_next_unit(this: &'static mut AccessoryShopChangeRoot, watching: bool, _optional_method: OptionalMethod) {
    if this.change_menu.kind == 0 {
        if this.change_menu.proc.desc_index < 4 { return; }
        change_character(this, true, watching);
    }
}
pub extern "C" fn accessory_change_root_previous_unit(this: &'static mut AccessoryShopChangeRoot, watching: bool, _optional_method: OptionalMethod) {
    if this.change_menu.kind == 0 {
        if this.change_menu.proc.desc_index < 4 { return; }
        change_character(this, false, watching);
    }
}
pub extern "C" fn accessory_change_root_key_on_start_watching(this: &'static mut AccessoryShopChangeRoot, _optional_method: OptionalMethod) {
    if this.change_menu.proc.desc_index < 4 { return; }
    if let Some(object) = GameObject::find("KeyHelpCamera") {
        if let Some(transform) = object.get_component_by_type::<RectTransform>() {
            let mut pos = transform.get_position();
            pos.y = 580.0;
            transform.set_position(pos);
        }
    }
    this.on_start_watching();
    this.equipment_menu.close();
}
pub extern "C" fn accessory_change_root_key_end_watching(this: &'static mut AccessoryShopChangeRoot, _optional_method: OptionalMethod) {
    if this.change_menu.proc.desc_index < 4 { return; }
    this.on_end_watching();
    this.equipment_menu.open();
    this.detail_info.show();
}
pub extern "C" fn accessory_change_root_key_on_show_ui(this: &'static mut AccessoryShopChangeRoot, _optional_method: OptionalMethod) {
    if !this.unit_name_object.is_null() {
        if let Some(anim) = this.unit_name_object.get_component_by_type::<Animator>() {
            if anim.get_bool("isClosed") { anim.play("Open"); }
        }
    }
    if let Some(object) = GameObject::find("KeyHelpCamera") {
        if let Some(transform) = object.get_component_by_type::<RectTransform>() {
            let mut pos = transform.get_position();
            pos.y = 580.0;
            transform.set_position(pos);
        }
    }
    this.equipment_menu.close();
}
pub(crate) fn accessory_menu_on_close_menu(this: &mut AccessoryShopChangeRoot, _optional_method: OptionalMethod) {
    this.on_request_close_menu();
    EquipmentBoxMode::CurrentProfilePage(EquipmentBoxPage::Assets).update();
}