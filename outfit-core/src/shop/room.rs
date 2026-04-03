use engage::{
    mess::Mess,
    combat::*,
    scene::ProcScene,
    unitinfo::*,
    hub::HubMiniMapMapMode,
    proc::{desc::ProcDesc, Bindable, ProcBoolMethod, ProcInst, ProcVoidMethod},
    manager::*,
    resourcemanager::ResourceManager,
    scene::SceneManager,
    gamedata::{Gamedata, GodData, assettable::AssetTableResult},
    unit::{UnitPool, Unit},
    gameuserdata::GameUserData,
    map::terrain::MapTerrainInfo,
    sequence::{
        gmap_sequence::GmapSequence,
        hub::HubSequence,
        hubaccessory::{HubAccessoryShopSequence, room::*},
    },
    sortie::SortieSequenceUnitSelect,
    titlebar::TitleBar,
    unityengine::*,
};
use engage::scene::LoadSceneMode;
use engage::util::{get_singleton_proc_instance};
use unity::prelude::*;
use unity::system::action::{SystemDelegate, Action};
use unity::system::List;
use crate::{get_outfit_data, print_asset_table_result, AssetType, CustomAssetMenu, EquipmentBoxMode, MenuMode, UnitAssetMenuData, FACIAL_STATES};
use engage::sequence::photograph::*;
use engage::ut::Ut;

#[unity::class("", ".<>c__DisplayClass31_0")]
pub struct HubAccessoryRoomAction31 {
    pub this: &'static HubAccessoryRoom,
    pub unit: &'static Unit,
    pub appearance: &'static CharacterAppearance,
}
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
    NoUpdate,
    Mount,
}
pub struct CustomHubAccessoryRoom;
impl CustomHubAccessoryRoom {
    pub fn create_bind<B: Bindable>(proc: &mut B) {
        let asset = UnitAssetMenuData::get();
        asset.is_shop_combat = GameUserData::get_sequence() != 4;
        asset.mode = MenuMode::Shop;
        asset.is_preview = true;
        HubAccessoryRoom::create_bind(proc, HubAccessoryRoomShop::Hub);
        if let Some(hub_accessory_room) = proc.get_child() {
            if let Some(hub_sequence) = get_singleton_proc_instance::<HubSequence>() {
                let mini_map = hub_sequence.get_mini_map();
                mini_map.set_mode(HubMiniMapMapMode::None);
                mini_map.hide_system_menu();
                mini_map.update();
            }
            if hub_accessory_room.name.is_some_and(|name| name.str_contains("HubAccessoryRoom")) {
                let descs = hub_accessory_room.descs.get_mut();
                descs[3] = ProcDesc::call(ProcVoidMethod::new(None, Self::additive_scene));
                descs[5] = ProcDesc::call(ProcVoidMethod::new(None, Self::init));
                descs[9] = ProcDesc::call(ProcVoidMethod::new(None, Self::main));
                descs[14] = ProcDesc::call(ProcVoidMethod::new(None, Self::exit));
                descs[15] = ProcDesc::wait_while_true(ProcBoolMethod::new(None, Self::is_character_loading));
                descs[16] = ProcDesc::call(ProcVoidMethod::new(None, Self::exit_other));
                descs[17] = ProcDesc::wait_while_true(ProcBoolMethod::new(None, Self::is_character_loading));
                descs[18] = ProcDesc::call(ProcVoidMethod::new(None, Self::exit_after));
            }
        }
    }
    pub extern "C" fn init(proc: &mut HubAccessoryRoom, _optional_method: OptionalMethod) {
        if Self::is_hub_solanel()  {
            proc.init();
            if let Some(scene) = HubSequence::get_instance().map(|v| v.scene_name){
                proc.return_scene_name = scene;
            }
        }
        else {
            let scene = SceneManager::get_scene_by_name("Hub_AccessoryRoom".into());
            SceneManager::set_active_scene(scene);
            proc.camera_pos = HubAccessoryRoomCamera::find_object(true);
            RenderManager::push_render_scale2(1.0);
        }
    }
    pub extern "C" fn main(proc: &mut HubAccessoryRoom, _optional_method: OptionalMethod) {
        proc.main();
        if let Some(hub_acc_shop) = proc.get_child() { hub_accessory_shop_sequence_edit(hub_acc_shop); }
    }
    pub fn on_dispose(proc: &mut ProcInst, _optional_method: OptionalMethod) {
        if let Some(parent) = proc.parent.as_ref() {
            if let Some(method) = parent.klass.get_virtual_method("OpenAnime") {
                let open_anime_all = unsafe { std::mem::transmute::<_, extern "C" fn(&ProcInst, &MethodInfo)>(method.method_info.method_ptr) };
                open_anime_all(parent, method.method_info);
            }
            if let Some(sortie) = SortieSequenceUnitSelect::get_instance() {
                sortie.disp_all();
                BackgroundManager::bind();
            }
        }
    }
    pub extern "C" fn additive_scene(proc: &mut HubAccessoryRoom, _optional_method: OptionalMethod) {
        let sequence = GameUserData::get_sequence();
        proc.disable_list.clear();
        match sequence {
            3|2 => {
                proc.return_scene_name = GameUserData::get_chapter().field;
                if sequence == 3 {
                    UnitInfo::set_visible_side(UnitInfoSide::Left, false);
                    if let Some(info) = MapTerrainInfo::get_instance() {
                        info.hide_all();
                    }
                }
            }
            4|5 => {
                if let Some(hub_sequence) = HubSequence::get_instance() { proc.return_scene_name = hub_sequence.scene_name; }
                proc.additive_scene();
                return;
            }
            6 => {
                if let Some(gmap) = GmapSequence::get_instance(){
                    gmap.map_info.close();
                    if let Some(scene_name) = gmap.scene_name {
                        proc.return_scene_name = scene_name;
                    }
                }
            }
            _ => { proc.additive_scene() }
        }
        let scene = SceneManager::get_scene_by_name(proc.return_scene_name);
        scene.get_root_game_objects().iter().for_each(|obj|{
            obj.set_active(false);
            proc.disable_list.add(obj);
        });
        proc.load_scene("Hub_AccessoryRoom", LoadSceneMode::Additive);
    }
    pub extern "C" fn exit(proc: &mut HubAccessoryRoom, _optional_method: OptionalMethod) {
        let menu_data = UnitAssetMenuData::get();
        menu_data.is_preview = false;
        menu_data.mode = MenuMode::Inactive;
        if Self::is_hub_solanel()  { proc.exit(); }
        else {
            if let Some(char) = proc.character.as_ref() { char.destroy(); }
            if let Some(camera) = proc.camera_pos.as_ref() { camera.destroy(); }
            if proc.disable_list.len() > 0 {
                let scene = SceneManager::get_scene_by_name(proc.return_scene_name);
                SceneManager::set_active_scene(scene);
                proc.disable_list.iter().for_each(|g|{ g.set_active(true); });
            }
            if GameUserData::get_sequence() == 3 { UnitInfo::set_visible_side(UnitInfoSide::Left, true); }
        }
    }
    pub extern "C" fn is_character_loading(proc: &mut HubAccessoryRoom, _optional_method: OptionalMethod) -> bool {
        if HubSequence::get_instance().is_some() { proc.is_character_loading() } else { ResourceManager::is_loading() }
    }
    pub extern "C" fn exit_other(proc: &mut HubAccessoryRoom, _optional_method: OptionalMethod) {
        if Self::is_hub_solanel()  { proc.exit_other(); }
    }
    pub extern "C" fn exit_after(proc: &mut HubAccessoryRoom, _optional_method: OptionalMethod) {
        if Self::is_hub_solanel()  { proc.exit_after(); }
    }
    pub extern "C" fn is_hub_solanel() -> bool { HubSequence::get_instance().is_some() }
}
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
    pub fn build_non_dress(this: &'static mut MyCharacterBuilderObject, _optional_method: OptionalMethod) {
        this.builder.attach_head_hair_and_weapons();
        this.builder.attach_dress();
        Self::set_unit_info_layer(this.builder);
    }
    pub fn replace_dress(builder: &'static mut CharacterBuilder, asset: &Il2CppString){
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
    pub fn build_dress(this: &'static mut MyCharacterBuilderObject, _optional_method: OptionalMethod) {
        if let Some(go) = this.builder.get_game_object().filter(|g| !g.is_null() ){
            go.get_components_in_children::<SkinnedMeshRenderer>(true).iter().for_each(|tr| {
                // let mesh = tr.get_name().to_string();
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


#[unity::class("Combat", "CharacterEffect")]
pub struct CharacterEffect {
    pub cp: &'static mut Character,
}

#[unity::hook("Combat", "CharacterEffect", "CreateBreak")]
pub fn create_break_effect(this: &mut CharacterEffect, method_info: OptionalMethod) {
    call_original!(this, method_info);
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
fn change_scaling(builder: &CharacterBuilder, result: Option<&mut AssetTableResult>) {
    let mut scale_values = [0.0; 16];
    if let Some(result) = result {
        for x in 0..16 { scale_values[x] = result.scale_stuff[x]; }
    }
    else {
        let preview = UnitAssetMenuData::get_preview();
        for x in 0..16 {
            scale_values[x] = preview.scale_preview[x] as f32 * 0.01 + 0.0001;
        }
    }
    if let Some(char_prop) = builder.get_component::<CharacterProportion>() {
        char_prop.proportion_parameters.scale_all = scale_values[0];
        char_prop.proportion_parameters.scale_head = scale_values[1];
        char_prop.proportion_parameters.scale_neck = scale_values[2];
        char_prop.proportion_parameters.scale_torso = scale_values[3];
        char_prop.proportion_parameters.scale_shoulders = scale_values[4];
        char_prop.proportion_parameters.scale_arms = scale_values[5];
        char_prop.proportion_parameters.scale_hands = scale_values[6];
        char_prop.proportion_parameters.scale_legs = scale_values[7];
        char_prop.proportion_parameters.scale_feet = scale_values[8];
        char_prop.proportion_parameters.volume_arms = scale_values[12] * scale_values[14];
        char_prop.proportion_parameters.volume_legs = scale_values[13] * scale_values[15];
        char_prop.proportion_parameters.volume_bust = scale_values[9];
        char_prop.proportion_parameters.volume_abdomen = scale_values[10];
        char_prop.proportion_parameters.volume_torso = scale_values[11];
        char_prop.proportion_parameters.flush();
        char_prop.commit_changes();
    }
}
fn force_load(result: Option<&mut AssetTableResult>, reload_type: ReloadType) {
    let result = result.or_else(||Some(UnitAssetMenuData::get_result())).unwrap();
    if let Some(p) = PhotographTopSequence::get_photograph_sequence(){
        crate::photo::update_character(p.dispos_manager.current_dispos_info, result);
    }
    else if let Some(room) = get_singleton_proc_instance::<HubAccessoryRoom>() {
        let pid = UnitAssetMenuData::get_shop_unit().map(|v| v.person.pid)
            .or_else(|| GodData::try_get_hash(UnitAssetMenuData::get_preview().person).map(|v| v.gid))
            .unwrap_or("PID_リュール".into());
        let hash = crate::new_result_get_hash_code(result, None);
        if hash == room.last_hash && reload_type != ReloadType::ForcedUpdate { return; }
        if reload_type == ReloadType::ForcedUpdate { room.destroy_current_char(); }
        room.last_hash = hash;
        let appearance = CharacterAppearance::create_from_result(result, 1);
        room.loading_appearance = Some(appearance);
        room.load_character(appearance, pid);
    }
    else if let Some((unit, info)) = UnitAssetMenuData::get_unit().zip(UnitInfo::get_instance()) {
        print_asset_table_result(result, 2);
        let char_model_window = &mut info.windows[0].unit_info_window_chara_model;
        let is_mount = reload_type == ReloadType::Mount;
        char_model_window.padding = if is_mount { 1 } else { 0 };
        let character = engage::sequence::talk::CharacterFactoryAsync::create_common(result, unit.person.pid, char_model_window.game_object, false, false, false);
        let display_class_104_klass = char_model_window.klass.get_nested_types()[4];
        let create_unit_action_object = display_class_104_klass.instantiate_as::<UnitInfoWindowCharaModelDisplayClass103>().unwrap();
        create_unit_action_object.this = char_model_window;
        create_unit_action_object.call_back = Some(character);
        let action = Action::instantiate().unwrap();
        action.ctor(Some(create_unit_action_object), display_class_104_klass.get_methods()[1]);
        action.method_ptr = create_char_model as _;
        create_unit_action_object.call_back.as_ref().map(|c| c.call_on_setup_done(action));
    }
}
pub fn hub_room_set_by_result(result: Option<&mut AssetTableResult>, reload_type: ReloadType) {
    let character =
        if UnitAssetMenuData::is_photo_graph() {
            PhotographTopSequence::get_photograph_sequence().and_then(|p| p.dispos_manager.current_dispos_info.m_character_cmp.as_ref())
        }
        else {
            get_singleton_proc_instance::<HubAccessoryRoom>().and_then(|v| v.character.as_ref())
                .or_else(|| UnitInfo::get_instance().map(|v| &v.windows[0].unit_info_window_chara_model.char))
        };
    if let Some(char) = character.filter(|v| v.op_implicit()){
        let builder = char.get_builder();
        match reload_type {
            ReloadType::Scale => {
                builder.appearance.proportion.flush();
                change_scaling(builder, result);
                UnitAssetMenuData::get().control.setup(false, true);
            }
            ReloadType::Dress => {
                let result = result.or_else(|| Some(UnitAssetMenuData::get_result())).unwrap();
                if builder.appearance.assets[1].name.is_some_and(|v| v.to_string() != result.dress_model.to_string()) {
                    MyCharacterBuilderObject::replace_dress(builder, result.dress_model);
                }
            }
            ReloadType::Head => {
                let result = result.or_else(|| Some(UnitAssetMenuData::get_result())).unwrap();
                if builder.appearance.assets[2].name.is_some_and(|v| v.to_string() != result.head_model.to_string()) {
                    MyCharacterBuilderObject::replace_head(builder, result.head_model);
                }
            }
            ReloadType::ColorScale => {
                let result = result.or_else(||Some(UnitAssetMenuData::get_result())).unwrap();
                builder.appearance.hair_color = result.unity_colors[0];
                builder.appearance.grad_color = result.unity_colors[1];
                builder.appearance.skin_color = result.unity_colors[2];
                builder.appearance.toon_shadow_color = result.unity_colors[3];
                builder.appearance.mask_color_100 = result.unity_colors[4];
                builder.appearance.mask_color_075 = result.unity_colors[5];
                builder.appearance.mask_color_050 = result.unity_colors[6];
                builder.appearance.mask_color_025 = result.unity_colors[7];
                if let Some(go) = builder.get_game_object().filter(|o| !o.is_null() ) { builder.appearance.modify_colors(go); }
            }
            ReloadType::Facial(increase) => {
                let len = 13;
                let v = UnitAssetMenuData::get().facial;
                let new_v = if increase { v + 1 + len} else { v + len - 1 } % len;
                char.play_facial(FACIAL_STATES[new_v].into());
                UnitAssetMenuData::get().facial = new_v;
            }
            _ => { force_load(result, reload_type); }
        }
    }
    else { force_load(result, reload_type); }
}

pub fn create_char_model(this: &mut UnitInfoWindowCharaModelDisplayClass103, _optional_method: OptionalMethod) {
    if let Some(character) = this.call_back.take() {
        this.this.destroy_chara_model();
        this.this.char = character;
        let char = this.this.create_chara_model(this.this.char);
        this.this.updater.is_request_to_offset = true;
        this.this.updater.late_update();
        this.this.updater.try_update_offset(char);
        char.play_facial(crate::FACIAL_STATES[UnitAssetMenuData::get().facial].into());
        let menu_data = UnitAssetMenuData::get();
        let trans = char.get_transform();
        /*
        let (x, y, z) = if this.this.padding & 1 != 0 {
            (0.0, 0.80, -2.25)
        }
        else if menu_data.model_pos[16] > -5.0 {
            let v = (menu_data.model_pos[16], menu_data.model_pos[17], menu_data.model_pos[18]);
            menu_data.model_pos[16] = -10.0;
            menu_data.model_pos[17] = -10.0;
            menu_data.model_pos[18] = -10.0;
            v
        }
        else { (menu_data.model_pos[0], menu_data.model_pos[1], menu_data.model_pos[2]) };
        */
        trans.set_position(menu_data.control.current_character.pos);
        trans.set_local_rotation(menu_data.control.current_character.rotation);
    }
}
#[repr(C)]
pub struct UnitInfoWindowCharaModelDisplayClass103 {
    klass: &'static Il2CppClass,
    monitor: u64,
    pub this: &'static mut UnitInfoWindowCharaModel,
    pub call_back: Option<&'static mut Character>,
}
pub fn hub_accessory_shop_sequence_edit(proc: &mut ProcInst) {
    let descs = proc.descs.get_mut();
    descs[3] = ProcDesc::call(ProcVoidMethod::new(None, start_sequence_hub_accessory_shop));
    descs[4] = ProcDesc::jump(4);
    descs[10] = ProcDesc::call(ProcVoidMethod::new(None, crate::shop::unitselect::create_accessory_unit_select));
    descs[20] = ProcDesc::call(ProcVoidMethod::new(None, crate::shop::unitselect::create_accessory_unit_select));
    descs[23] = ProcDesc::jump(6); // Jump to End
    descs[24] = ProcDesc::jump(6); // Jump to End
    descs[26] = ProcDesc::call(ProcVoidMethod::new(None, create_accessory_change_menu));    // Edited AccessoryShopMenu
}

pub extern "C" fn start_sequence_hub_accessory_shop(this: &mut HubAccessoryShopSequence, _optional_method: OptionalMethod) {
    this.shop_menu_result = 1;
    if let Some(unit) = UnitPool::get_first(9, 0) {
        UnitAssetMenuData::set_unit(unit);
        HubAccessoryRoom::set_unit(unit, None, true, false);
        this.unit = Some(unit);
    }
}
pub extern "C" fn create_accessory_change_menu(this: &mut HubAccessoryShopSequence, _optional_method: OptionalMethod) {
    this.create_accessory_shop_change_menu();
    CustomAssetMenu::init(this.change_root.change_menu, true);
    crate::shop::change_root::edit_accessory_root_change_unit(this.change_root.change_root_proc);
    let data = UnitAssetMenuData::get();
    if data.god_mode {
        let god_name = GodData::try_get_hash(data.preview.person).map(|v| format!("{} ({})", Mess::get(v.mid), Mess::get("MID_H_INFO_Param_Correction_God")).into()).unwrap_or(Mess::get("MPID_Unknown"));
        this.change_root.unit_name.set_text(god_name, true);
    }
    EquipmentBoxMode::CurrentProfile.change_equipment_box(this.change_root.equipment_menu);
    TitleBar::hide_footer();
}

pub extern "C" fn hub_accessory_init(hub_room: &mut HubAccessoryRoom, _optional_method: OptionalMethod) {
    if get_singleton_proc_instance::<HubSequence>().is_some_and(|s| s.scene_name.str_contains("Hub_Solanel")) {
        hub_room.init();
    }
    else {
        let scene = SceneManager::get_scene_by_name("Hub_AccessoryRoom".into());
        SceneManager::set_active_scene(scene);
        hub_room.camera_pos = HubAccessoryRoomCamera::find_object(true); // find_object_of_type::<HubAccessoryRoomCamera>(get_type_(room_camera_type, None), None);
        RenderManager::push_render_scale2(1.0);
    }
}