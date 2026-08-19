use engage::{
    app::{
        titlebar::*, proc::*, procinst::*, IProcSceneSequence_1Methods, fade::{Fade, Fade_Layer as FadeLayer},
        ISingletonProcInst_1Methods, IHubAccessoryShopSequenceMethods,
        hubaccessoryroom::*, HubAccessoryShopSequence, HubAccessoryRoomCamera, IHubAccessoryShopSequence,
        photographsequence::*, IPhotographDisposInfo, IPhotographDisposManager,
        gmapsequence::{GmapSequence, IGmapSequence}, hubsequence::*, IHubMiniMapMethods, IGmapMapInfoContentMethods,
        accessoryshopchangeroot::*, accessoryequipmentinfo::*, accessoryshopchangemenu::*,
        ProcVoidMethod, ProcBoolMethod, ProcDesc,
        ISingletonClass_1Methods,  IGameUserDataMethods,
        IHubPlayerControllerMethods, IHubLocatorGroupMethods,
        BasicMenuContent, BasicMenu, IBasicMenuMethods,
        AccessoryShopChangeMenuContent, AccessoryDetailInfoWindow, AccessoryShopTopMenu_Result2,
        IGodDataMethods, IStructData_1Methods, IPersonDataMethods,
        IUnitInfo, IUnitInfoWindowCharaModel, IUnitInfo_Window,
        IUnitMethods,
        RenderManager, ResourceManager_2,
    },
    List_1Ext,
    ProcBoolMethodExt, ProcExt, ProcVoidMethodExt,
    system::collections::generic::IList_1Methods,
    unity_engine::{
        IGameObjectMethods,
        scene_management::{LoadSceneMode, SceneManager},
    },
    tm_pro::{ITMP_Text, ITMP_TextMethods},
    prelude::{Cast, Object},
};
use unity::{field_set_value_at_offset, ClassIdentity, FromIlInstance, Il2CppString, IlNull, IntPtr, SystemObject};
use crate::{CustomAssetMenu, EquipmentBoxMode, MenuMode, OutfitMenuKind, UnitAssetMenuData};
use crate::data::change_root::create_accessory_shop_change_root_proc;
use crate::data::unitselect::create_accessory_unit_select;

pub struct CustomHubAccessoryRoom;
impl CustomHubAccessoryRoom {
    pub fn create_bind(proc: impl Into<ProcInst>) {
        let sequence = engage::app::GameUserData::get_instance().get_sequence().value;
        if sequence < 4 { return; }
        if sequence == 4 {
            let hub_sequence = HubSequence::get_instance();
            if !hub_sequence.is_null() {
                let scene = hub_sequence.m_scene_name().to_rust_string();
                if scene.contains("GodRoom") { return; }
                let map = hub_sequence.get_mini_map();
                map.set_mode(engage::app::HubMiniMap_MapMode::none());
                map.hide_system_menu();
                map.update();
            }
        }
        let asset = UnitAssetMenuData::get();
        asset.is_shop_combat = sequence != 4;
        asset.mode = MenuMode::Shop;
        asset.is_preview = true;
        let room = HubAccessoryRoom::new(HubAccessoryRoom_Shop::hub());
        let room_obj = Object::from_il_instance(room.as_instance());
        let descs = [
            Fade::black_out(0.25, FadeLayer::current()),
            Fade::fade_wait(FadeLayer::current()),
            Proc::call_method(ProcVoidMethod::new(room_obj, IntPtr::from(HubAccessoryRoom::open_title_method_info()))),
            Proc::call_method(ProcVoidMethod::from_fn(room.into(), Self::additive_scene).unwrap()),
            Proc::r#yield(),
            Proc::call_method(ProcVoidMethod::from_fn(room.into(), Self::init).unwrap()),
            Proc::r#yield(),
            Fade::black_in(0.25, FadeLayer::current()),
            Fade::fade_wait(FadeLayer::current()),
            Proc::call_method(ProcVoidMethod::from_fn(room.into(), Self::main).unwrap()),
            Fade::black_out(0.25, FadeLayer::current()),
            Fade::fade_wait(FadeLayer::current()),
            Proc::call_method(ProcVoidMethod::new(room_obj, IntPtr::from(HubAccessoryRoom::un_additive_scene_method_info()))),
            Proc::r#yield(),
            Proc::call_method(ProcVoidMethod::from_fn(room.into(), Self::exit).unwrap()),    // 14
            Proc::wait_while_true_2(ProcBoolMethod::from_fn(room.into(), Self::is_character_loading).unwrap()),
            Proc::call_method(ProcVoidMethod::from_fn(room.into(), Self::exit_other).unwrap()),
            Proc::wait_while_true_2(ProcBoolMethod::from_fn(room.into(), Self::is_character_loading).unwrap()),
            Proc::call_method(ProcVoidMethod::from_fn(room.into(), Self::exit_after).unwrap()),
            Proc::r#yield(),
            Fade::black_in(0.25, FadeLayer::current()),
            Fade::fade_wait(FadeLayer::current()),
            Proc::call_method(ProcVoidMethod::from_fn(room.into(), Self::restore_menu).unwrap()),
            Proc::end(),
        ];
        let arr = unity::Array::<ProcDesc>::new(<ProcDesc as ClassIdentity>::class().raw(), descs.len()).unwrap();
        for (i, d) in descs.iter().enumerate() { arr.set(i, *d); }
        room.create_bind(proc, arr, "CustomHubAccessoryRoom");
    }
    pub extern "C" fn init(proc: HubAccessoryRoom, _: unity::OptionalMethod) {
        let hub = HubSequence::get_instance();
        if !hub.is_null() {
            let group = hub.get_locator_group();
            if !group.is_null() { group.save_accessory(); }
            let controller = hub.get_player();
            if !controller.is_null() { controller.save_accessory(); }
        }
        let scene = SceneManager::get_scene_by_name("Hub_AccessoryRoom");
        SceneManager::set_active_scene(scene);
        let camera = engage::unity_engine::Object_2::find_object_of_type::<HubAccessoryRoomCamera>();
        if !camera.is_null() { proc.set_camera_pos(camera); }
        RenderManager::push_render_scale_2(1.0);
    }
    pub extern "C" fn main(proc: HubAccessoryRoom, _: unity::OptionalMethod) {
        let shop_sequence = HubAccessoryShopSequence::new();
        let descs = [
            Proc::label(0), // Label 0
            Proc::call_method(ProcVoidMethod::new(shop_sequence.into(), IntPtr::from(HubAccessoryShopSequence::load_resources_method_info()))),
            Proc::wait_while_true_2(ProcBoolMethod::new(shop_sequence.into(), IntPtr::from(HubAccessoryShopSequence::is_loading_resources_method_info()))),
            Proc::call_method(ProcVoidMethod::from_fn(shop_sequence.into(), start_sequence_hub_accessory_shop).unwrap()),
            Proc::label(1), // Label 1  UnitSelectMenu
            Proc::call_method(ProcVoidMethod::from_fn(shop_sequence.into(), create_accessory_unit_select).unwrap()),
            Proc::call_method(ProcVoidMethod::new(shop_sequence.into(), IntPtr::from(HubAccessoryShopSequence::destroy_shop_unit_select_menu_method_info()))),
            Proc::jump_true_3(ProcBoolMethod::from_fn(shop_sequence.into(), jump_to_menu_check).unwrap(), 2),  // Jump to AccessoryChange
            Proc::jump_true_3(ProcBoolMethod::from_fn(shop_sequence.into(), jump_to_exit).unwrap(), 3),
            Proc::label(2), // Label 2  AccessoryChangeMenu
            Proc::call_method(ProcVoidMethod::from_fn(shop_sequence.into(), create_accessory_change_menu).unwrap()),
            Proc::call_method(ProcVoidMethod::from_fn(shop_sequence.into(), destroy).unwrap()),
            Proc::jump(1),  // Jump back to UnitSelectMenu
            Proc::label(3), // Label 3  Exit
            Proc::call_method(ProcVoidMethod::new(shop_sequence.into(), IntPtr::from(HubAccessoryShopSequence::end_sequence_method_info()))),
            Proc::end(),
        ];
        let arr = unity::Array::<ProcDesc>::new(<ProcDesc as ClassIdentity>::class().raw(), descs.len()).unwrap();
        for (i, d) in descs.iter().enumerate() { arr.set(i, *d); }
        shop_sequence.create_bind(proc, arr, "CustomHubAccessoryShopSequence");
    }
    pub extern "C" fn restore_menu(proc: HubAccessoryRoom, _: unity::OptionalMethod) {
        if let Some(menu) = proc.m_super().try_cast::<BasicMenu>(){ menu.open_anime_all(); }
    }
    pub extern "C" fn additive_scene(proc: HubAccessoryRoom, _: unity::OptionalMethod) {
        let user_data = engage::app::GameUserData::get_instance();
        let sequence = user_data.get_sequence().value;
        match sequence {
            4|5 => {
                let hub = HubSequence::get_instance();
                if !hub.is_null() { field_set_value_at_offset::<Il2CppString>(proc, 0x90, hub.m_scene_name()); }
            }
            6 => {
                let gmap = GmapSequence::get_instance();
                if !gmap.is_null() {
                    gmap.m_map_info().close();
                    field_set_value_at_offset::<Il2CppString>(proc, 0x90, gmap.get_scene_name());
                }
            }
            _ => { return; }
        }
        let disable_list = proc.disable_list();
        disable_list.clear();
        let mut scene = SceneManager::get_scene_by_name(proc.get_return_scene_name());
        scene.get_root_game_objects().iter().for_each(|o|{
            if o.get_active_self() {
                o.set_active(false);
                disable_list.add(o);
            }
        });
        proc.load_scene_2("Hub_AccessoryRoom".into(), LoadSceneMode::additive());
    }
    pub extern "C" fn exit(proc: HubAccessoryRoom, _: unity::OptionalMethod) {
        let menu_data = UnitAssetMenuData::get();
        menu_data.is_preview = false;
        menu_data.mode = MenuMode::Inactive;
        RenderManager::pop_render_scale();
        let character = proc.m_character();
        if !character.is_null() { engage::unity_engine::Object_2::destroy_2(character); }
        let camera = proc.get_camera_pos();
        if !camera.is_null() { engage::unity_engine::Object_2::destroy_2(camera); }
        let scene = SceneManager::get_scene_by_name(proc.get_return_scene_name());
        SceneManager::set_active_scene(scene);
        let disable_list = proc.disable_list();
        disable_list.iter().for_each(|g|{ g.set_active(true); });
        disable_list.clear();
        let hub = HubSequence::get_instance();
        if !hub.is_null() {
            let player_controller = hub.get_player();
            if !player_controller.is_null() {
                player_controller.restore_accessory();
            }
        }
    }
    pub extern "C" fn is_character_loading(proc: HubAccessoryRoom, _: unity::OptionalMethod) -> bool {
        crate::menu::proc::OutfitSequence::character_loading(proc.into(), None)
    }
    pub extern "C" fn exit_other(_: HubAccessoryRoom, _: unity::OptionalMethod) {
        let hub = HubSequence::get_instance();
        if !hub.is_null() {
            let group = hub.get_locator_group();
            if !group.is_null() {
                group.set_active(true);
                group.restore_accessory();
            }
        }
    }
    pub extern "C" fn exit_after(_: HubAccessoryRoom, _: unity::OptionalMethod) {
        let hub = HubSequence::get_instance();
        if !hub.is_null() {
            let group = hub.get_locator_group();
            if !group.is_null() { group.reset_look_at(); }
            let controller = hub.get_player();
            if !controller.is_null() { controller.init_look_at_target(); }
        }
    }
    pub fn get_character() -> Option<engage::combat::Character> {
        if UnitAssetMenuData::is_photo_graph() {
            crate::photo::get_photosequence().map(|p| p.m_dispos_manager().m_current_dispos_info().m_character_cmp())
        }
        else {
            let room = HubAccessoryRoom::get_instance();
            if !room.is_null() { Some(room.m_character()) } else {
                let info = engage::app::UnitInfo::get_instance();
                Some(info.m_windows().get(0).m_unit_info_window_chara_model().m_chara())
            }
        }
    }
}
extern "C" fn jump_to_menu_check(proc: HubAccessoryShopSequence, _: unity::OptionalMethod) -> bool {
    proc.m_shop_menu_result() == AccessoryShopTopMenu_Result2::change()
}
extern "C" fn jump_to_exit(proc: HubAccessoryShopSequence, _: unity::OptionalMethod) -> bool {
    proc.m_shop_menu_result() == AccessoryShopTopMenu_Result2::end()
}
pub extern "C" fn destroy(this: HubAccessoryShopSequence, _: unity::OptionalMethod) {
    this.destroy_accessory_shop_change_menu();
}
pub extern "C" fn start_sequence_hub_accessory_shop(this: HubAccessoryShopSequence, _: unity::OptionalMethod) {
    let unit = engage::app::UnitPool::get_first(9u32, 0);
    if !unit.is_null() {
        this.set_m_unit(unit);
        UnitAssetMenuData::set_unit(unit);
        let hub = engage::app::GameUserData::get_instance().get_sequence().value == 4;
        let data = UnitAssetMenuData::get();
        data.mode = MenuMode::Shop;
        data.is_shop_combat = !hub;
        data.is_hub = hub;
        HubAccessoryRoom::set_unit(unit, engage::app::AccessoryData::null(), true, false);
    }
}
pub extern "C" fn create_accessory_change_menu(this: HubAccessoryShopSequence, _: unity::OptionalMethod) {
    if let Some(root) = accessory_shop_change_create_bind(this, AccessoryShopChangeRoot_ReturnEventHandler::null()) {
        EquipmentBoxMode::CurrentProfile.change_equipment_box(root.m_accessory_equipment_info_window());
        this.set_m_accessory_shop_change_root(root);
        TitleBar::get_instance().hide_footer();
    }
}
fn accessory_shop_change_create_bind(proc: impl Into<ProcInst> + Copy, return_handler: AccessoryShopChangeRoot_ReturnEventHandler) -> Option<AccessoryShopChangeRoot> {
    let canvas = BasicMenuContent::get_canvas();
    let canvas_transform = canvas.get_transform();
    let x = ResourceManager_2::instantiate_2("UI/Hub/Shop/Prefabs/ShopAccChangeRoot", canvas_transform);
    if !x.is_null() {
        let root = x.get_component::<AccessoryShopChangeRoot>();
        if !root.is_null() {
            create_accessory_shop_change_root_proc(proc, root);
            let menu_object = root.m_menu_object();
            let menu_content = menu_object.get_component::<AccessoryShopChangeMenuContent>();
            let menu = CustomAssetMenu::new(menu_content);
            let request_close = AccessoryShopChangeMenu_RequestCloseEventHandler::new(root.into(), AccessoryShopChangeRoot::on_request_close_menu_method_info().into());
            menu.set_m_request_close_event_handler(request_close);
            root.set_m_accessory_shop_change_menu(unsafe { menu.cast() });
            root.set_m_return_event_handler(return_handler);
            menu.create_bind(proc, menu.create_default_desc(), "OutfitAccessoryRoomMenu");
            let unit_name = root.m_unit_name();
            let data = UnitAssetMenuData::get();
            if !unit_name.is_null() {
                if data.god_mode {
                    let god = engage::app::GodData::try_get_from_hash(data.preview.person);
                    if !god.is_null() {
                        unit_name.set_m_text(format!("{} ({})", engage::app::Mess::get(god.get_mid()), engage::app::Mess::get("MID_H_INFO_Param_Correction_God")).into());
                    }
                    else { unit_name.set_m_text(engage::app::Mess::get("MPID_Unknown")); }
                }
                else {
                    let person = engage::app::PersonData::try_get_from_hash(UnitAssetMenuData::get().preview.person);
                    if !person.is_null() {
                        let unit = engage::app::UnitPool::get_from_person(person, false);
                        if !unit.is_null() { unit_name.set_text(unit.get_name()); }
                        else { unit_name.set_text(engage::app::Mess::get(person.get_name())); }
                    }
                }
            }
            let equipment_info = root.m_equipment_info_window_object();
            if !equipment_info.is_null() {
                let window = equipment_info.get_component::<AccessoryEquipmentInfo>();
                if !window.is_null() {
                    crate::build_equipment_window(window, true);
                    root.set_m_accessory_equipment_info_window(window);
                    window.m_cursor_object().set_active(false);
                }
            }
            let detail_info = root.m_detail_info_window_object();
            if !detail_info.is_null() {
                let detail = detail_info.get_component::<AccessoryDetailInfoWindow>();
                if !detail.is_null() { root.set_m_accessory_detail_info_window(detail); }
            }
            HubAccessoryRoom::set_view_mode(HubAccessoryRoom_ViewMode{value: 1});
            let result = data.unit_select.get_result(!data.is_shop_combat);
            crate::hub_room_set_by_result(Some(result), crate::ReloadType::All);
            crate::start_key_help(OutfitMenuKind::AccessoryShop);
            Some(root)
        }
        else { None }
    }
    else { None }
}