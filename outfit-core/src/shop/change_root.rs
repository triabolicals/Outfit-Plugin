use crate::{data::room::hub_room_set_by_result, room::ReloadType};
pub use crate::menu::*;
use engage::{
    app::{
        accessoryshopchangerootproc::*,
        AccessoryShopChangeRoot,
        IAccessoryDetailInfoWindowMethods, IAccessoryEquipmentInfoMethods,
        IAccessoryShopChangeMenu, IAccessoryShopChangeRoot, IAccessoryShopChangeRootMethods,
        IBasicMenuMethods,
        IProcInst, IProcInstMethods
    },
    unity_engine::{IGameObjectMethods, IAnimatorMethods, ITransformMethods},
    tm_pro::ITMP_TextMethods
};
use unity::{Cast, FromIlInstance, IlNull, IntPtr};

pub fn create_accessory_shop_change_root_proc(proc: impl Into<engage::app::ProcInst>, root: AccessoryShopChangeRoot) -> AccessoryShopChangeRootProc{
    let change_root = AccessoryShopChangeRootProc::instantiate().unwrap();
    let next_unit = AccessoryShopChangeRootProc_ChangeUnitToNextEventHandler::new(root.into(), accessory_change_root_next_unit_method_info().into());
    let previous_unit = AccessoryShopChangeRootProc_ChangeUnitToPrevEventHandler::new(root.into(), accessory_change_root_previous_unit_method_info().into());
    let watching = AccessoryShopChangeRootProc_StartWatchingEventHandler::new(root.into(), accessory_change_root_key_on_start_watching_method_info().into());
    let end_watching = AccessoryShopChangeRootProc_EndWatchingEventHandler::new(root.into(), accessory_change_root_key_end_watching_method_info().into());
    let on_show_ui = AccessoryShopChangeRootProc_ShowUIEventHandler::new(root.into(), accessory_change_root_key_on_show_ui_method_info().into());
    let hide_ui = AccessoryShopChangeRootProc_HideUIEventHandler::new(root.into(), IntPtr::from(AccessoryShopChangeRoot::on_hide_ui_method_info()));
    
    change_root.set_m_show_ui_event_handler(on_show_ui);
    change_root.set_m_start_watching_event_handler(watching);
    change_root.set_m_end_watching_event_handler(end_watching);
    change_root.set_m_change_unit_to_prev_event_handler(previous_unit);
    change_root.set_m_change_unit_to_next_event_handler(next_unit);
    change_root.set_m_hide_ui_event_handler(hide_ui);
    change_root.set_m_key_help_all_animator(root.m_key_help_all_animator());
    change_root.set_m_key_help_all_object(root.m_key_help_all_object());
    change_root.create_bind_no_desc(proc);
    root.set_m_accessory_shop_change_root_proc(change_root);
    change_root
}
fn change_character(this: AccessoryShopChangeRoot, next: bool, watching: bool) {
    UnitAssetMenuData::commit();
    let data = UnitAssetMenuData::get();
    data.unit_select.change(next);
    if let Some(selected) = data.unit_select.get_selected() {
        UnitAssetMenuData::set_by_hash(selected.hash);
        let result = selected.get_result(data.is_hub);
        let asset_menu = unsafe { this.m_accessory_shop_change_menu().cast::<CustomAssetMenu>() };
        asset_menu.rebuild_menu(MainShop, false);
        hub_room_set_by_result(Some(result), ReloadType::All);
        if let Some(name) = selected.get_name() { this.m_unit_name().set_text_2(name, true); }
        engage::app::GameSound::post_event("Chara_Change", engage::combat::Character::null());
    }
    if watching {
        this.m_menu_object().set_active(false);
        this.m_detail_info_window_object().set_active(false);
        this.m_accessory_shop_change_menu().set_input_disable(true);
        let help = this.m_key_help_all_object();
        if !help.is_null() {
            let transform = help.get_transform();
            let mut pos = transform.get_position();
            pos.y = 580.0;
            transform.set_position(pos);
        }
    }
    else { this.m_accessory_detail_info_window().show(); }
}
#[unity::callback]
pub extern "C" fn accessory_change_root_next_unit(this: AccessoryShopChangeRoot, watching: bool, _: unity::OptionalMethod) {
    let menu = this.m_accessory_shop_change_menu();
    if menu.m_kind().value == 0 {
        if this.m_accessory_shop_change_menu().m_desc_index() < 4 { return; }
        change_character(this, true, watching);
    }
}
#[unity::callback]
pub extern "C" fn accessory_change_root_previous_unit(this: AccessoryShopChangeRoot, watching: bool, _: unity::OptionalMethod) {
    let menu = this.m_accessory_shop_change_menu();
    if  menu.m_kind().value  == 0 {
        if this.m_accessory_shop_change_menu().m_desc_index() < 4 { return; }
        change_character(this, false, watching);
    }
}
#[unity::callback]
pub extern "C" fn accessory_change_root_key_on_start_watching(this: AccessoryShopChangeRoot, _: unity::OptionalMethod) {
    if this.m_accessory_shop_change_menu().m_desc_index() < 4 { return; }
    this.on_start_watching();
    this.m_accessory_equipment_info_window().close();
}
#[unity::callback]
pub extern "C" fn accessory_change_root_key_end_watching(this: AccessoryShopChangeRoot, _: unity::OptionalMethod) {
    if this.m_accessory_shop_change_menu().m_desc_index() < 4 { return; }
    this.on_end_watching();
    this.m_accessory_equipment_info_window().open();
    this.m_accessory_detail_info_window().hide();
}
#[unity::callback]
pub extern "C" fn accessory_change_root_key_on_show_ui(this: AccessoryShopChangeRoot, _: unity::OptionalMethod) {
    if !this.m_unit_name_object().is_null() {
        let anim = this.m_unit_name_object().get_component::<engage::unity_engine::Animator>();
        if !anim.is_null() { if anim.get_bool("isClosed") { anim.play_2("Open"); } }
    }
    let help = this.m_key_help_all_object();
    if !help.is_null() {
        let transform = help.get_transform();
        let mut pos = transform.get_position();
        pos.y = 580.0;
        transform.set_position(pos);
    }
    this.m_accessory_equipment_info_window().close();
}