use engage::{gamesound::GameSound};
use unity::prelude::*;
use crate::{data::room::hub_room_set_by_result, room::ReloadType};
pub use crate::menu::*;
use engage_il2cpp::app::accessoryshopchangerootproc::*;
use engage_il2cpp::app::{IAccessoryShopChangeRoot, IProcInstMethods};
use unity2::{FromIlInstance, SystemObject};

pub fn create_accessory_shop_change_root_proc(proc: impl Into<engage_il2cpp::app::ProcInst>, root: engage_il2cpp::app::AccessoryShopChangeRoot) -> AccessoryShopChangeRootProc{
    let change_root = AccessoryShopChangeRootProc::instantiate().unwrap();
    let obj = engage_il2cpp::system::Object::from_il_instance(change_root.as_instance());
    let next_unit = AccessoryShopChangeRootProc_ChangeUnitToNextEventHandler::new(obj, );
    let previous_unit = AccessoryShopChangeRootProc_ChangeUnitToPrevEventHandler::new(obj, );
    let watching = AccessoryShopChangeRootProc_StartWatchingEventHandler::new(obj, );
    let end_watching = AccessoryShopChangeRootProc_EndWatchingEventHandler::new(obj, );
    let on_show_ui = AccessoryShopChangeRootProc_ShowUIEventHandler::new(obj, );
    let hide_ui = AccessoryShopChangeRootProc_HideUIEventHandler::new(obj, );
    change_root.set_m_show_ui_event_handler(on_show_ui);
    change_root.set_m_start_watching_event_handler(watching);
    change_root.set_m_end_watching_event_handler(end_watching);
    change_root.set_m_change_unit_to_prev_event_handler(previous_unit);
    change_root.set_m_change_unit_to_next_event_handler(next_unit);
    change_root.set_m_hide_ui_event_handler(hide_ui);
    change_root.set_m_key_help_all_animator(root.m_key_help_all_animator());
    change_root.set_m_key_help_all_object(root.m_key_help_all_object());
    change_root.create_bind_no_desc(proc);
    change_root
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