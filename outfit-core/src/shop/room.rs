use engage::{
    app::{
        assettable::{AssetTable_Result, IAssetTable_ResultMethods},
        titlebar::*, proc::*, procinst::*, IProcSceneSequence_1Methods, fade::{Fade, Fade_Layer as FadeLayer},
        IUnitInfoWindowCharaUpdater, ISingletonProcInst_1Methods, IHubAccessoryShopSequenceMethods,
        hubaccessoryroom::*, HubAccessoryShopSequence, HubAccessoryRoomCamera, IHubAccessoryShopSequence,
        photographsequence::*,  IPhotographDisposInfo, IPhotographDisposManager,
        gmapsequence::{GmapSequence, IGmapSequence}, hubsequence::*, IHubMiniMapMethods,IGmapMapInfoContentMethods,
        accessoryshopchangeroot::*, accessoryequipmentinfo::*, accessoryshopchangemenu::*,
        ProcVoidMethod, ProcBoolMethod, ProcDesc,
        ISingletonClass_1Methods, IChapterDataMethods, IGameUserDataMethods,
        IHubPlayerControllerMethods, IHubLocatorGroupMethods,
        BasicMenuContent, BasicMenu, IBasicMenuMethods,
        AccessoryShopChangeMenuContent, AccessoryDetailInfoWindow, AccessoryShopTopMenu_Result2,
        IGodDataMethods, IStructData_1Methods, IPersonDataMethods,
        IUnitInfo, IUnitInfoWindowCharaModel, IUnitInfoWindowCharaModelMethods, IUnitInfoWindowCharaUpdaterMethods, IUnitInfo_Window,
        IUnitMethods,
        UnitInfoWindowCharaModel, talk3_d::CharacterFactoryAsync_2,
        RenderManager, ResourceManager_2,
    },
    system::object::*,
    List_1Ext, ProcBoolMethodExt, ProcExt, ProcVoidMethodExt,
    system::collections::generic::IList_1Methods,
    unity_engine::{
        IComponentMethods, IGameObjectMethods,
        scene_management::{LoadSceneMode, SceneManager},
        IObject_2Methods, IRendererMethods, ITransformMethods
    },
    combat::{
        characterappearance::*,
        ICharacterAssetForm, ICharacterAssetT_1Methods,
        ICharacterMethods,
        ICharacterProportion, ICharacterProportionMethods,
        IProportionParameters, IProportionParametersMethods,
    },
    tm_pro::{ITMP_Text, ITMP_TextMethods},
    prelude::{Cast, Object},
};
use unity::{field_set_value_at_offset, ClassIdentity, FromIlInstance, Il2CppString, IlNull, IntPtr, SystemObject};
use crate::{get_outfit_data, get_result_color, get_result_scale_f32, AssetType, CustomAssetMenu, EquipmentBoxMode, MenuMode, Mount, OutfitMenuKind, UnitAssetMenuData, FACIAL_STATES};
use crate::data::change_root::create_accessory_shop_change_root_proc;
use crate::data::unitselect::create_accessory_unit_select;

#[derive(PartialEq, Clone, Copy)]
pub enum ReloadType {
    All,
    ColorScale,
    Body,
    Dress,
    Hair,
    Accessories(usize),
    ForcedUpdate,
    Head,
    Scale,
    Facial(bool),
    FacialPreview(usize),
    NoUpdate,
    Mount,
    HairAcc,
    HeadAcc,
}
pub struct CustomHubAccessoryRoom;
impl CustomHubAccessoryRoom {
    pub fn create_bind(proc: impl Into<ProcInst>) {
        let sequence = engage::app::GameUserData::get_instance().get_sequence().value;
        if sequence < 4 { return; }
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
        if sequence != 6 {
            let hub_sequence = HubSequence::get_instance();
            if !hub_sequence.is_null() {
                let map = hub_sequence.get_mini_map();
                map.set_mode(engage::app::HubMiniMap_MapMode::none());
                map.hide_system_menu();
                map.update();
            }
        }
    }
    pub extern "C" fn init(proc: HubAccessoryRoom, _: unity::OptionalMethod) {
        let hub = HubSequence::get_instance();
        if !hub.is_null() {
            let group = hub.m_hub_locator_group();
            if !group.is_null() { group.save_accessory(); }
            let controller = hub.m_hub_player_controller();
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
        let mut scene = SceneManager::get_scene_by_name(proc.get_return_scene_name());
        let disable_list = proc.disable_list();
        disable_list.clear();
        scene.get_root_game_objects().iter().for_each(|o|{
            o.set_active(false);
            disable_list.add(o);
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
        if disable_list.count() > 0 { disable_list.iter().for_each(|g|{ g.set_active(true); }); }
        let hub = HubSequence::get_instance();
        if !hub.is_null() {
            let player_controller = hub.m_hub_player_controller();
            if !player_controller.is_null() { player_controller.restore_accessory(); }
        }
    }
    pub extern "C" fn is_character_loading(proc: HubAccessoryRoom, _: unity::OptionalMethod) -> bool {
        if !HubSequence::get_instance().is_null() { proc.is_character_loading() } else { ResourceManager_2::is_loading() }
    }
    pub extern "C" fn exit_other(_: HubAccessoryRoom, _: unity::OptionalMethod) {
        let hub = HubSequence::get_instance();
        if !hub.is_null() {
            let group = hub.m_hub_locator_group();
            if !group.is_null() {
                group.set_active(true);
                group.restore_accessory();
            }
        }
    }
    pub extern "C" fn exit_after(_: HubAccessoryRoom, _: unity::OptionalMethod) {
        let hub = HubSequence::get_instance();
        if !HubSequence::get_instance().is_null() {
            let group = hub.m_hub_locator_group();
            if !group.is_null() { group.reset_look_at(); }
            let controller = hub.m_hub_player_controller();
            if !controller.is_null() { controller.init_look_at_target(); }
        }
    }
}
extern "C" fn jump_to_menu_check(proc: HubAccessoryShopSequence, _: unity::OptionalMethod) -> bool {
    proc.m_shop_menu_result() == AccessoryShopTopMenu_Result2::change()
}
extern "C" fn jump_to_exit(proc: HubAccessoryShopSequence, _: unity::OptionalMethod) -> bool {
    proc.m_shop_menu_result() == AccessoryShopTopMenu_Result2::end()
}

/*
#[repr(C)]
pub struct MyCharacterBuilderObject {
    klass: &'static Il2CppClass,
    monitor: u64,
    pub builder: &'static mut CharacterBuilder,
    pub old_objects: &'static mut List<Il2CppString>,
}

impl MyCharacterBuilderObject {
    pub fn replace_head(builder: &'static mut CharacterBuilder, asset: &Il2CppString) {
        if !builder.head.is_null() {
            builder.head.get_transform().set_parent(None);
            builder.head.destroy();
        }
        let display_class_104_klass = UnitInfoWindowCharaModel::class().get_nested_types()[4];
        let create_unit_action_object = display_class_104_klass.instantiate_as::<MyCharacterBuilderObject>().unwrap();
        builder.appearance.assets[2].set_name(asset);
        create_unit_action_object.builder = builder;
        let action = Action::instantiate().unwrap();
        action.ctor(Some(create_unit_action_object), display_class_104_klass.get_methods()[1]);
        action.method_ptr = MyCharacterBuilderObject::build_non_dress as _;
        create_unit_action_object.builder.appearance.assets[2].load_async(Some(action));
    }
    pub fn replace_hair(builder: &'static mut CharacterBuilder, asset: &Il2CppString) {
        if !builder.hair.is_null() {
            builder.hair.get_transform().set_parent(None);
            builder.hair.destroy();
        }
        let display_class_104_klass = UnitInfoWindowCharaModel::class().get_nested_types()[4];
        let create_unit_action_object = display_class_104_klass.instantiate_as::<MyCharacterBuilderObject>().unwrap();
        builder.appearance.assets[3].set_name(asset);
        create_unit_action_object.builder = builder;
        let action = Action::instantiate().unwrap();
        action.ctor(Some(create_unit_action_object), display_class_104_klass.get_methods()[1]);
        action.method_ptr = MyCharacterBuilderObject::build_non_dress as _;
        create_unit_action_object.builder.appearance.assets[3].load_async(Some(action));
    }
    fn set_unit_info_layer(this: &CharacterBuilder) {
        if let Some(go) = this.get_game_object().filter(|g| !g.is_null())  {
            let builder_transform = go.get_transform();
            if let Some(parent) = builder_transform.get_parent() {
                let parent_name = parent.get_name().to_string();
                if parent_name.contains("UnitInfoWindowCharaModel") {
                    if let Some(char) = this.get_component::<Character>() {
                        let layer = UnitInfo::get_instance().unwrap().windows[0].unit_info_window_chara_model.camera_object.get_layer();
                        if let Some(go) = char.get_game_object().filter(|g| !g.is_null()) {
                            Ut::set_layer_recursively2(go, layer);
                        }
                    }
                }
            }
            this.appearance.modify_colors(go);
        }
    }
    pub fn build_non_dress(this: &'static mut MyCharacterBuilderObject, _optional_method: unity::OptionalMethod) {
        this.builder.attach_head_hair_and_weapons();
        this.builder.attach_dress();
        if let Some(go) = this.builder.get_game_object() { this.builder.appearance.modify_colors(go); }
        Self::set_unit_info_layer(this.builder);
    }
    pub fn replace_dress(builder: engage::combat::CharacterBuilder, asset: unity::Il2CppString){
        let go = builder.get_game_object();
        if go.is_null() { return; }
        let appearance = builder.appearance();
        let list = List_1::<unity::Il2CppString>::new();
        list.add("c_trans".into());
        list.add("HoldWeapon".into());
        list.add("LookTarget".into());
        for xx in [0, 2, 3, 4, 5, 12, 13, 14, 15, 16, 17, 18, 19] {
            let asset = appearance.assets().get(xx);
            if asset.is_null() { continue; }
            if !asset.is_ready() { continue; }
            let go = builder.get_go(asset);
            if go.is_null() { continue; }
            go.get_components_in_children_3::<engage::unity_engine::Transform>(true).iter().for_each(|t|{
                list.add(t.get_name());
            });
        };

        if let Some(go) = builder.get_game_object().filter(|g| !g.is_null()) {
            let list = List::<Il2CppString>::with_capacity(256).unwrap();
            list.add("c_trans".into());
            list.add("HoldWeapon".into());
            list.add("LookTarget".into());
            for xx in [0, 2, 3, 4, 5, 12, 13, 14, 15, 16, 17, 18, 19] {
                if builder.appearance.assets[xx].name.is_none() || !builder.appearance.assets[xx].is_ready() { continue; }
                if let Some(go) = builder.appearance.assets[xx].get_asset() {
                    go.get_components_in_children::<Transform>(true).iter().for_each(|x| {
                        list.add(x.get_name());
                    })
                }
            };
            let go_transform = go.get_transform();
            if let Some(c_trans) = Kaneko::find_in_children(go_transform, "c_trans".into()) {
                let keep_trans = list.iter().map(|v| v.to_string()).collect::<Vec<String>>();
                c_trans.get_components_in_children_gen::<Transform>(true).iter()
                    .for_each(|t|{
                        let xx = t.get_name().to_string();
                        if !keep_trans.contains(&xx) && !xx.contains("Camera") {
                            t.set_parent(None);
                            if let Some(obj) = t.get_game_object(){ obj.destroy(); }
                        }
                    });
            }
            let display_class_104_klass = UnitInfoWindowCharaModel::class().get_nested_types()[4];
            let create_unit_action_object = display_class_104_klass.instantiate_as::<MyCharacterBuilderObject>().unwrap();
            builder.appearance.assets[1].set_name(asset);
            create_unit_action_object.builder = builder;
            create_unit_action_object.old_objects = list;
            let action = Action::instantiate().unwrap();
            action.ctor(Some(create_unit_action_object), display_class_104_klass.get_methods()[1]);
            action.method_ptr = MyCharacterBuilderObject::build_dress as _;
            create_unit_action_object.builder.appearance.assets[1].load_async(Some(action));
        }
    }
    pub fn build_dress(this: &'static mut MyCharacterBuilderObject, _optional_method: unity::OptionalMethod) {
        if let Some(go) = this.builder.get_game_object().filter(|g| !g.is_null() ){
            go.get_components_in_children::<SkinnedMeshRenderer>(true).iter().for_each(|tr| {
                if let Some(parent) = tr.get_transform().get_parent() {
                    if parent.get_name().to_string() == "meshGP" {
                        if let Some(go) = tr.get_game_object() { go.destroy(); }
                    }
                }
            });
            this.builder.dress_ut = Some(DressUtility::new(go.get_transform()));
            this.builder.attach_dress();
            Self::set_unit_info_layer(this.builder);
            this.builder.dress_ut = None;
        }
    }
}
 */
/*
pub fn break_effect(this: &mut CharacterEffect){

if let Some(unit) = this.cp.get_game_status().unit.as_ref() {
    let engaged = unit.is_engaging();
    if let Some(data) = UnitAssetMenuData::get_unit_data(unit).filter(|s| s.get_active_flag(engaged) & 32 != 0){
        let new_body = if let Some(profile) = data.profile.get( data.profile_index(engaged) as usize ) { profile.break_body } else { 0 };
        let builder = this.cp.get_builder();
        if let Some(body) = builder.appearance.assets[1].name.as_ref() {
            let hashcode = body.get_hash_code();
            let db = get_outfit_data();
            let new_asset =
                if db.hashes.male_u.contains(&hashcode) && db.hashes.male_u.contains(&new_body){ db.try_get_asset(AssetType::Body, new_body) }
                else if db.hashes.female_u.contains(&hashcode) && db.hashes.female_u.contains(&new_body){ db.try_get_asset(AssetType::Body, new_body) }
                else { None };

            if let Some(body) = new_asset { MyCharacterBuilderObject::replace_dress(builder, body.into()); }
        }
    }
}
}
 */
fn change_scaling(builder: engage::combat::CharacterBuilder, result: Option<AssetTable_Result>) {
    let char_prop = builder.get_component_2::<engage::combat::CharacterProportion>();
    if !char_prop.is_null() {
        let mut scale_values = [0.0; 16];
        if let Some(result) = result {
            for x in 0..16 { scale_values[x] = get_result_scale_f32(result, x) + 0.001; }
        }
        else {
            let preview = UnitAssetMenuData::get_preview();
            for x in 0..16 { scale_values[x] = preview.scale_preview[x] as f32 * 0.01 + 0.01; }
        }
        let prop = char_prop.proportion_parameters();
        prop.set_scale_all(scale_values[0]);
        prop.set_scale_head(scale_values[1]);
        prop.set_scale_neck(scale_values[2]);
        prop.set_scale_torso(scale_values[3]);
        prop.set_scale_shoulders(scale_values[4]);
        prop.set_scale_arms(scale_values[5]);
        prop.set_scale_hands(scale_values[6]);
        prop.set_scale_legs(scale_values[7]);
        prop.set_scale_feet(scale_values[8]);
        prop.set_volume_arms(scale_values[12] * scale_values[14]);
        prop.set_volume_legs(scale_values[13] * scale_values[15]);
        prop.set_volume_bust(scale_values[9]);
        prop.set_volume_abdomen(scale_values[10]);
        prop.set_volume_torso(scale_values[11]);
        prop.flush();
        char_prop.commit_changes();
    }
}
fn force_load(result: Option<AssetTable_Result>, reload_type: ReloadType) {
    let result = result.or_else(||Some(UnitAssetMenuData::get_result())).unwrap();
    update_result_for_preview(result);
    let room = HubAccessoryRoom::get_instance();
    if !room.is_null() {
        if reload_type == ReloadType::ForcedUpdate { room.destroy_current_char(); }
        let appearance = CharacterAppearance::create_from_result(result, 1);
        room.set_m_loading_appearance(appearance);
        room.load_character(appearance, "PID_リュール");
    }
    else {
        let p = engage::app::PhotographTopSequence::get_instance();
        if !p.is_null() {
            if let Some(photograph) = p.get_child().try_cast::<PhotographSequence>() {
                crate::photo::update_character(photograph.m_dispos_manager().m_current_dispos_info(), result);
            }
        }
        else {
            let info = engage::app::UnitInfo::get_instance();
            // if let Some(unit) = UnitAssetMenuData::get_unit().filter(|u| !u.is_null() ) {
                let char_model_window = info.m_windows().get(0).m_unit_info_window_chara_model();
                let character = CharacterFactoryAsync_2::create_common(result, "PID_不明", char_model_window.m_game_object(), false, false, false);
                let create_character_object = CreateUnitInfoModel::instantiate().unwrap();
                create_character_object.set_character(character);
                create_character_object.set_unit_info_window(char_model_window);
                let action = engage::system::Action::new(create_character_object.into(), create_char_model_method_info().into());
                character.call_on_setup_done(action);
          //  }
        }
    }
}
pub fn hub_room_set_by_result(result: Option<AssetTable_Result>, reload_type: ReloadType) {
    let character = {
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
    };
    if let Some(char) = character.filter(|v| !v.is_null()){
        let builder = char.get_builder();
        let appearance = builder.appearance();
        match reload_type {
            ReloadType::Scale => {
                appearance.proportion().flush();
                change_scaling(builder, result);
                UnitAssetMenuData::get().control.setup(false, true);
            }
            /*
            ReloadType::Dress => {
                let result = result.unwrap_or(UnitAssetMenuData::get_result());
                let dress = appearance.assets().get(1);
                let name = dress.get_name();
                if !name.is_null() && !result.get_dress_model().is_null() {
                    let n1 = name.to_rust_string();
                    let n2 = result.get_dress_model().to_rust_string();
                    if n1 != n2 {
                        MyCharacterBuilderObject::replace_dress(builder, result.dress_model)
                    }
                }
            }
            ReloadType::Head => {
                UnitAssetMenuData::get_preview().update = 2;
                let result = result.or_else(|| Some(UnitAssetMenuData::get_result())).unwrap();
                let dress = appearance.assets().get(1);
                let name = dress.get_name();
                if !name.is_null() && !result.get_head_model().is_null() {
                    let n1 = name.to_rust_string();
                    let n2 = result.get_dress_model().to_rust_string();
                    if n1 != n2 {
                        MyCharacterBuilderObject::replace_head(builder, result.get_head_model());
                    }
                }
            }
             */
            ReloadType::ColorScale => {
                let result = result.or_else(||Some(UnitAssetMenuData::get_result())).unwrap();
                appearance.set_hair_color(get_result_color(result, 0));
                appearance.set_grad_color(get_result_color(result, 1));
                appearance.set_skin_color(get_result_color(result, 2));
                appearance.set_toon_shadow_color(get_result_color(result, 3));
                appearance.set_mask_color100(get_result_color(result, 4));
                appearance.set_mask_color075(get_result_color(result, 5));
                appearance.set_mask_color050(get_result_color(result, 6));
                appearance.set_mask_color025(get_result_color(result, 7));
                let go = builder.get_game_object();
                if !go.is_null() { appearance.modify_colors(go); }
            }
            ReloadType::Facial(increase) => {
                let len = FACIAL_STATES.len();
                let v = UnitAssetMenuData::get().facial;
                let new_v = if increase { v + 1 + len} else { v + len - 1 } % len;
                char.play_facial(FACIAL_STATES[new_v].0);
                UnitAssetMenuData::get().facial = new_v;
                EquipmentBoxMode::CurrentProfile.update();
            }
            ReloadType::HairAcc => {
                let go = builder.get_game_object();
                if !go.is_null() { hair_acc(go, UnitAssetMenuData::get_flag() & 16 != 0); }
            }
            ReloadType::HeadAcc => {
                let go = builder.get_game_object();
                if !go.is_null() { head_acc(go, UnitAssetMenuData::get_flag() & 64 != 0); }
            }
            ReloadType::FacialPreview(index) => { char.play_facial(FACIAL_STATES[index].0); }
            _ => { force_load(result, reload_type); }
        }
    }
    else { force_load(result, reload_type); }
}

#[unity::inject(namespace = "App", name = "CreateUnitInfoModel", parent=Object)]
pub struct CreateUnitInfoModel {
    pub unit_info_window: UnitInfoWindowCharaModel,
    pub character: engage::combat::Character,
}
#[unity::callback]
pub fn create_char_model(this: CreateUnitInfoModel, _: unity::OptionalMethod) {
    let character = this.character();
    let unit_info_window = this.unit_info_window();
    if !character.is_null() && !unit_info_window.is_null() {
        let old_char = unit_info_window.m_chara();
        unit_info_window.delete_chara_model_2(old_char);
        unit_info_window.set_m_chara(character);
        let char = unit_info_window.create_chara_model_2(character);
        let update = unit_info_window.m_chara_updater();
        update.set_m_is_request_to_offset(true);
        update.late_update();
        update.try_update_offset(char);
        let trans = char.get_transform();
        let menu_data = UnitAssetMenuData::get();
        trans.set_position(menu_data.control.current_character.pos);
        trans.set_local_rotation(menu_data.control.current_character.rotation);
    }
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
            hub_room_set_by_result(Some(result), ReloadType::All);
            crate::start_key_help(OutfitMenuKind::AccessoryShop);
            Some(root)
        }
        else { None }
    }
    else { None }
}
pub fn hair_acc(go: engage::unity_engine::GameObject, enable: bool){
    if go.is_null() { return; }
    for name in ["meshHairGP", "c_spine1_jnt"]{
        let t = engage::combat::Kaneko::find_in_children(go.get_transform(), name);
        if !t.is_null() {
            let go = t.get_game_object();
            if let Some(arr) = crate::get_skin_mesh_renderers(go) {
                arr.iter()
                    .map(|r| unsafe { r.cast::<engage::unity_engine::SkinnedMeshRenderer>() })
                    .for_each(|r| {
                        let name = r.get_name().to_rust_string();
                        if (name.contains("_Acc") && name.starts_with("h")) || name.starts_with("acc") { r.set_enabled(!enable); }
                    });
            }
            return;
        }
    }
}
pub fn head_acc(go: engage::unity_engine::GameObject, enable: bool){
    if !go.is_null() {
        if let Some(arr) = crate::get_skin_mesh_renderers(go) {
            arr.iter()
                .map(|r| unsafe { r.cast::<engage::unity_engine::SkinnedMeshRenderer>()})
                .for_each(|r| {
                    let go = r.get_game_object();
                    let p = go.get_transform().get_parent();
                    if !p.is_null() {
                        let name = p.get_name().to_rust_string();
                        if name.contains("Head") {
                            let go_name = r.get_name().to_rust_string();
                            if go_name.starts_with("Make") || go_name.starts_with("Acc_") {
                                r.set_enabled(!enable);
                            }
                        }
                    }
                })
        }
    }
}
fn update_result_for_preview(result: AssetTable_Result) {
    if let Some((kind, hash)) = UnitAssetMenuData::get_preview().preview_asset.take() {
        let db = get_outfit_data();
        if let Some(asset) = db.try_get_asset(kind, hash){
            match kind {
                AssetType::AOC(_) => {
                    crate::anim::AnimData::remove(result, true, true);
                    result.set_body_anim(asset.as_str());
                    return;
                }
                AssetType::Body => { result.set_dress_model(asset.as_str()); }
                AssetType::Rig => { result.set_body_model(asset.as_str()); }
                AssetType::Head => { result.set_head_model(asset.as_str()); }
                AssetType::Hair => {
                    crate::apply_result_hair(asset, result);
                    result.replace(engage::app::AssetTable_Modes::combat());
                }
                AssetType::Acc(kind) => {
                    if asset.contains("Msc0AT") { result.set_left_hand(asset.as_str()); }
                    else {
                        let acc_locator = crate::ACC_LOC[kind as usize];
                        result.commit_8(crate::new_asset_table_accessory(asset.as_str(), acc_locator));
                        result.replace(engage::app::AssetTable_Modes::combat());
                    }
                }
                AssetType::Mount(k) => {
                    result.get_body_anims().clear();
                    let dress = db.get_dress_gender(result.get_dress_model());
                    let gender = if db.get_dress_gender(result.get_dress_model()) == engage::app::Gender::female() { "F" } else { "M" };
                    result.set_ride_dress_model(asset.as_str());
                    result.set_ride_model(Mount::from_i32(1 + k as i32).get_default_asset(true));
                    match k {
                        0 => {
                            let anim = format!("Cav0B{}-No1_c000_N", gender);
                            result.set_body_anim(anim.as_str());
                        }
                        1 => {
                            let anim = format!("Cav2C{}-No1_c000_N", gender);
                            result.set_body_anim(anim.as_str());
                        }
                        2 => {
                            let anim = format!("Wng2D{}-No1_c000_N", gender);
                            result.set_body_anim(anim.as_str());
                        }
                        3 => {
                            if dress == engage::app::Gender::male() { result.set_dress_model("uBody_Wng0EF_c000"); }
                            result.set_body_anim("Wng0EF-No1_c000_N");
                        }
                        4 => {
                            let anim = format!("Wng1F{}-No1_c000_N", gender);
                            result.set_body_anim(anim.as_str());
                        }
                        _ => {} // result.body_anims.add(format!("Com0A{}-No1_c000_N", gender).into()); }
                    }
                    return;
                }
                _ => { return; }
            }
            if UnitAssetMenuData::is_unit_info() && kind != AssetType::Body {
                result.set_body_anim(if db.get_dress_gender(result.get_dress_model()) == engage::app::Gender::male() { "AOC_Hub_Hum0M" } else { "AOC_Hub_Hum0F" });
            }
        }
    }
}
