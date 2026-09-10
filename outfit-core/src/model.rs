pub use engage::{
    app::{
        assettable::*, unit::*,
        hubaccessoryroom::*,
        unitinfo::*, unitinfowindowcharamodel::*, unitinfowindowcharaupdater::*,
    },
    unity_engine::component::*,
    combat::{character::*, characterbuilder::*, characterappearance::*, Kaneko},
};
use engage::{
    app::{
        Ut,
        photographsequence::*, IProcInstMethods,
        IPhotographDisposInfo, IPhotographDisposManager, IPhotographSequence, ISingletonProcInst_1Methods,
        JobData, talk3_d::CharacterFactoryAsync_2
    },
    system::object::*,
    combat::{
        ICharacterAssetForm, characterproportion::*, proportionparameters::*, characterjoint::*,
    },
    unity_engine::{
        animator::*, camera::*, material::*, gameobject::*, transform::*,
        IRendererMethods, SkinnedMeshRenderer, Screen, Vector3, IObject_2Methods,
    },
    unity_engine::{Color, ParticleSystemRenderer, Renderer}
};
use super::*;
use unity::Cast;
use crate::{EquipmentBoxMode, Mount, UnitAssetData, UnitAssetMenuData, FACIAL_STATES};
const RENDERER_FILTER: [&'static str; 3] = ["EyeHL", "EyeLash", "Mucous"];
#[derive(PartialEq, Clone, Copy)]
pub enum ReloadType {
    All,
    ColorScale,
    HeadColor,
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
    AOC,
}

fn change_scaling(builder: CharacterBuilder, result: Option<AssetTable_Result>) {
    let char_prop = builder.get_component_2::<CharacterProportion>();
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
        let p = PhotographTopSequence::get_instance();
        if !p.is_null() {
            if let Some(photograph) = p.get_child().try_cast::<PhotographSequence>() {
                photo::update_character(photograph.m_dispos_manager().m_current_dispos_info(), result);
            }
        }
        else {
            let info = UnitInfo::get_instance();
            let char_model_window = info.m_windows().get(0).m_unit_info_window_chara_model();
            let character = CharacterFactoryAsync_2::create_common(result, "PID_不明", char_model_window.m_game_object(), false, false, false);
            let create_character_object = CreateUnitInfoModel::instantiate().unwrap();
            create_character_object.set_character(character);
            let mount = il2str(result.get_ride_dress_model()).is_some_and(|v| Mount::determine_mount(v) != Mount::None);
            create_character_object.set_reset_animation(reload_type == ReloadType::AOC);
            create_character_object.set_mount(mount);
            create_character_object.set_unit_info_window(char_model_window);
            let action = engage::system::Action::new(create_character_object.into(), create_char_model_method_info().into());
            character.call_on_setup_done(action);
        }
    }
}
pub fn hub_room_set_by_result(result: Option<AssetTable_Result>, reload_type: ReloadType) {
    let character = {
        if UnitAssetMenuData::is_photo_graph() {
            photo::get_photosequence().map(|p| p.m_dispos_manager().m_current_dispos_info().m_character_cmp())
        }
        else {
            let room = HubAccessoryRoom::get_instance();
            if !room.is_null() { Some(room.m_character()) } else {
                let info = UnitInfo::get_instance();
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
            ReloadType::HeadColor => {
                let go = builder.get_game_object();
                if !go.is_null() {
                    apply_colors_2(appearance, go);
                }
                // apply_preview_head_hair_color(appearance, go); }
            }
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
                EquipmentBoxMode::set_cursor(Some(4));
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
pub fn update_class_change_person(unit: Unit, job: JobData) {
    if unit.is_null() || job.is_null() { return; }
    let info = UnitInfo::get_instance();
    let char_model_window = info.m_windows().get(0).m_unit_info_window_chara_model();
    unit.set_job(job);
    let result = AssetTable_Result::get_for_unit_info(unit);
    let character = CharacterFactoryAsync_2::create_common(result, "PID_不明", char_model_window.m_game_object(), false, false, false);
    let create_character_object = CreateUnitInfoModel::instantiate().unwrap();
    create_character_object.set_character(character);
    create_character_object.set_unit_info_window(char_model_window);
    create_character_object.set_is_job(true);
    let action = engage::system::Action::new(create_character_object.into(), create_char_model_method_info().into());
    character.call_on_setup_done(action);
}
#[unity::inject(namespace = "App", name = "CreateUnitInfoModel", parent=Object)]
pub struct CreateUnitInfoModel {
    pub unit_info_window: UnitInfoWindowCharaModel,
    pub character: Character,
    pub is_job: bool,
    pub mount: bool,
    pub reset_animation: bool,
}
#[unity::callback]
pub fn create_char_model(this: CreateUnitInfoModel, _: OptionalMethod) {
    let character = this.character();
    let unit_info_window = this.unit_info_window();
    if !character.is_null() && !unit_info_window.is_null() {
        let old_char = unit_info_window.m_chara();
        let body_states = AnimatorStates::new(old_char.get_body_animator());
        let face_states = AnimatorStates::new(old_char.get_face_animator());
        unit_info_window.delete_chara_model_2(old_char);
        unit_info_window.set_m_chara(character);
        let char = unit_info_window.create_chara_model_2(character);
        let update = unit_info_window.m_chara_updater();
        update.set_m_is_request_to_offset(true);
        update.try_update_offset(char);
        let camera = UnitInfo::get_face_camera_component(UnitInfo_Side::left());
        if !this.is_job() {
            update.late_update();
            let trans = char.get_transform();
            let menu_data = UnitAssetMenuData::get();
            let ride = Kaneko::find_in_children(char.get_transform(), "lookAt_ride_loc");
            if !ride.is_null(){
                trans.set_local_scale(Vector3{x: 0.60, y: 0.60, z: 0.60});
                let camera = UnitInfo::get_face_camera_component(UnitInfo_Side::left());
                let h = Screen::get_height() as f32;
                let w = Screen::get_width() as f32;
                let head_world_1 = ride.get_position();
                let mut head_cam_pos = camera.world_to_screen_point_2(head_world_1);
                head_cam_pos.x = 0.45 * w;
                head_cam_pos.y = 0.70 * h;
                let head_world_2 = camera.screen_to_world_point_2(head_cam_pos);
                let x_adjust = head_world_2.x - head_world_1.x;
                let y_adjust = head_world_2.y - head_world_1.y;
                let mut character_trans = trans.get_position();
                character_trans.x += x_adjust;
                character_trans.y += y_adjust;
                character_trans.z = -1.9;
                trans.set_position(character_trans);
                menu_data.control.set_mounted(trans, ride);
                trans.set_local_rotation(menu_data.control.current_character.rotation);
            }
            else {
                let joint = char.get_component_2::<CharacterJoint>();
                if !joint.is_null() {
                    let look_at_loc = joint.get_look_at_loc();
                    if !look_at_loc.is_null() {
                        let look_at_loc = look_at_loc.get_position();
                        let camera_trans = camera.get_transform();
                        let mut camera_pos = camera_trans.get_position();
                        camera_pos.y = look_at_loc.y + 0.4;
                        camera_trans.set_position(camera_pos);
                    }
                }
                menu_data.control.mount = false;
                trans.set_position(menu_data.control.current_character.pos);
                trans.set_local_rotation(menu_data.control.current_character.rotation);
            }
        }
        if !this.reset_animation() {
            if let Some(body) = body_states{ body.set_animator(char.get_body_animator()); }
            if let Some(face) = face_states { face.set_animator(char.get_face_animator()); }
        }
    }
}
pub struct AnimatorStates { pub states: Vec<(i32, i32, f32)>}
impl AnimatorStates {
    pub fn new(animator: Animator) -> Option<AnimatorStates> {
        if animator.is_null() { return None;}
        let n_layers = animator.get_layer_count();
        let mut states = vec![];
        for i in 0..n_layers {
            let state = animator.get_current_animator_state_info(i);
            states.push((i, state.m_full_path, Kaneko::fixed_time(state)));
        }
        Some(Self { states })
    }
    pub fn set_animator(&self, animator: Animator) {
        if animator.is_null() { return; }
        self.states.iter().for_each(|&(index, hash, fixed_time)|{ animator.play_in_fixed_time_4(hash, index, fixed_time) });
    }
}
fn exist_in_hierarchy(transform: Transform, s: &str) -> bool{
    let mut search = transform;
    loop {
        let name = search.get_name().to_rust_string();
        if name.starts_with(s) { return true }
        else { search = search.get_parent(); }
        if search.is_null() { return false }
    }
}
pub fn hair_acc(go: GameObject, enable: bool){
    if go.is_null() { return; }
    for name in ["meshHairGP", "c_spine1_jnt"]{
        let t = Kaneko::find_in_children(go.get_transform(), name);
        if !t.is_null() {
            let go = t.get_game_object();
            go.get_components_in_children_3::<SkinnedMeshRenderer>(true).iter()
                .for_each(|r| {
                    let name = r.get_name().to_rust_string();
                    if (name.contains("_Acc") && name.starts_with("h")) || name.starts_with("acc") { r.set_enabled(!enable); }
                });
        }
        return;
    }
}
pub fn head_acc(go: GameObject, enable: bool){
    if !go.is_null() {
        go.get_components_in_children_3::<SkinnedMeshRenderer>(true)
            .iter()
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
fn update_result_for_preview(result: AssetTable_Result) {
    if let Some((kind, hash)) = UnitAssetMenuData::get_preview().preview_asset.take() {
        let db = get_outfit_data();
        if let Some(asset) = db.try_get_asset(kind, hash){
            match kind {
                AssetType::AOC(_) => {
                    anim::AnimData::remove(result, true, true);
                    result.set_body_anim(asset.as_str());
                    return;
                }
                AssetType::Body => { result.set_dress_model(asset.as_str()); }
                AssetType::Rig => { result.set_body_model(asset.as_str()); }
                AssetType::Head => { result.set_head_model(asset.as_str()); }
                AssetType::Hair => {
                    apply_result_hair(asset, result);
                    result.replace(AssetTable_Modes::combat());
                }
                AssetType::Acc(kind) => {
                    if asset.contains("Msc0AT") { result.set_left_hand(asset.as_str()); }
                    else {
                        let acc_locator = ACC_LOC[kind as usize];
                        result.commit_8(new_asset_table_accessory(asset.as_str(), acc_locator));
                        result.replace(AssetTable_Modes::combat());
                    }
                }
                AssetType::Mount(k) => {
                    let body_anims = result.get_body_anims();
                    body_anims.clear();
                    let dress = db.get_dress_gender(result.get_dress_model());
                    let gender = if db.get_dress_gender(result.get_dress_model()) == engage::app::Gender::female() { "F" } else { "M" };
                    result.set_ride_dress_model(asset.as_str());
                    result.set_ride_model(Mount::from_i32(1 + k as i32).get_default_asset(true));
                    match k {
                        0 => {
                            let anim = format!("Cav0B{}-No1_c000_N", gender);
                            result.set_body_anim(anim.as_str());
                            body_anims.add(format!("Com0B{}-No1_c000_N", gender).into());
                            body_anims.add(anim.as_str().into());
                        }
                        1 => {
                            let anim = format!("Cav2C{}-No1_c000_N", gender);
                            result.set_body_anim(anim.as_str());
                            body_anims.add(anim.as_str().into());
                        }
                        2 => {
                            let anim = format!("Wng2D{}-No1_c000_N", gender);
                            result.set_body_anim(anim.as_str());
                            body_anims.add(anim.as_str().into());
                        }
                        3 => {
                            if dress == engage::app::Gender::male() { result.set_dress_model("uBody_Wng0EF_c000"); }
                            result.set_body_anim("Wng0EF-No1_c000_N");
                            body_anims.add("Wng0EF-No1_c000_N".into());
                        }
                        4 => {
                            let anim = format!("Wng1F{}-No1_c000_N", gender);
                            result.set_body_anim(anim.as_str());
                            body_anims.add(anim.as_str().into());
                        }
                        _ => {}
                    }
                    return;
                }
                _ => { return; }
            }
            result.set_body_anim(if db.get_dress_gender(result.get_dress_model()) == engage::app::Gender::male() { "AOC_Hub_Hum0M" } else { "AOC_Hub_Hum0F" });
        }
    }
}
pub fn get_eyes_colors(this: CharacterAppearance, go: GameObject) {
    if this.is_null() || go.is_null() { return;}
    let renderer = go.get_components_in_children_3::<Renderer>(true);
    let stencil = CharacterAppearance::s_stencil_value() + 1.0;
    if stencil != 127.0 || stencil.is_nan() { CharacterAppearance::set_s_stencil_value(1.0); }
    let data = UnitAssetMenuData::get();
    let update = if data.is_preview { data.preview.update } else { 0 };
    if update & 1 != 0 {
        for x in 0..4 {
            data.preview.original_color[56 + x] = 0;
            data.preview.original_color[x] = 0;
            data.preview.original_color[x + 4] = 0;
        }
    }
    if update & 2 != 0 {
        for i in 0..4 { data.preview.original_color[8+i] = 0; }
        for i in 0..24 { data.preview.original_color[32+i] =0; }
    }
    let list = List_1::<Material>::new_2(32);
    renderer.iter().for_each(|r| {
        if !r.is_null() && !r.is_direct_subclass_of::<ParticleSystemRenderer>() {
            let transform = r.get_transform();
            let renderer_name = r.get_name().to_rust_string();
            if exist_in_hierarchy(transform, "meshHead") && !RENDERER_FILTER.iter().any(|v| renderer_name.contains(v)) {
                let materials = Ut::get_instance_materials(r);
                materials.iter().for_each(|m| {
                    let material_name = m.get_name().to_rust_string();
                    list.add(m);
                    material_set_color_from_appearance(m, this);
                    if update & 2 != 0 {
                        if !material_name.contains("Eye2") && material_name.starts_with("MtEye"){
                            UnitAssetData::EYE_COLOR.iter()
                                .enumerate()
                                .for_each(|(i, x)| {
                                    let color = m.get_color_2(*x);
                                    data.preview.original_color[32 + i * 4] = (color.r * 255.0) as u8;
                                    data.preview.original_color[33 + i * 4] = (color.g * 255.0) as u8;
                                    data.preview.original_color[34 + i * 4] = (color.b * 255.0) as u8;
                                    data.preview.original_color[35 + i * 4] = 1;
                                });
                        }
                        if material_name.starts_with("MtSkin") || material_name.starts_with("MtuSkin"){
                            let color = m.get_color_3(CharacterAppearance::hash_base_color());
                            data.preview.original_color[8] = (color.r * 255.0) as u8;
                            data.preview.original_color[9] = (color.g * 255.0) as u8;
                            data.preview.original_color[10] = (color.b * 255.0) as u8;
                            data.preview.original_color[11] = 1;
                        }
                    }
                })
            }
            else {
                let shared = r.get_shared_materials();
                if this.has_material_to_modify(shared) {
                    let materials = Ut::get_instance_materials(r);
                    materials.iter().for_each(|m| {
                        if !m.is_null() {
                            list.add(m);
                            material_set_color_from_appearance(m, this);
                            let name = m.get_name().to_rust_string();
                            if update & 1 != 0 {
                                if name.contains("MtHair2") || name.contains("MtOdd"){
                                    let color = m.get_color_3(CharacterAppearance::hash_base_color());
                                    data.preview.original_color[56] = (color.r * 255.0) as u8;
                                    data.preview.original_color[57] = (color.g * 255.0) as u8;
                                    data.preview.original_color[58] = (color.b * 255.0) as u8;
                                    data.preview.original_color[59] = 1;
                                }
                                else if name.starts_with("MtHair") && !name.contains("Hair2"){
                                    ["_BaseColor", "_GradationColor"].into_iter().enumerate().for_each(|(i, x)|{
                                        let color = m.get_color_2(x);
                                        data.preview.original_color[4 * i] = (color.r * 255.0) as u8;
                                        data.preview.original_color[4 * i + 1] = (color.g * 255.0) as u8;
                                        data.preview.original_color[4 * i + 2] = (color.b * 255.0) as u8;
                                        data.preview.original_color[4 * i + 3] = 1;
                                    });
                                }
                            }
                        }
                    })
                }
            }
        }
    });
    data.preview.update = 0;
    // println!("Instanced Materials: {}", list.count());
    this.set_m_instanced_materials(list);
}
fn material_set_color_from_appearance(m: Material, app: CharacterAppearance) {
    let name = m.get_name().to_rust_string();
    let grad_color = app.grad_color();
    if m.has_property(CharacterAppearance::hash_stencil_group()) { m.set_float_2(CharacterAppearance::hash_stencil_group(), CharacterAppearance::s_stencil_value()); }
    if AssetTable::has_color(grad_color) && m.has_property(CharacterAppearance::hash_gradation_color()) && grad_color.a > 0.0 { m.set_color_3(CharacterAppearance::hash_gradation_color(), grad_color); }
    if AssetTable::has_color(app.toon_shadow_color()) && m.has_property(CharacterAppearance::hash_toon_shadow_color()) { m.set_color_3(CharacterAppearance::hash_toon_shadow_color(), app.toon_shadow_color()); }
    if AssetTable::has_color(app.mask_color100()) && m.has_property(CharacterAppearance::hash_mask_color100()) { m.set_color_3(CharacterAppearance::hash_mask_color100(), app.mask_color100()); }
    if AssetTable::has_color(app.mask_color075()) && m.has_property(CharacterAppearance::hash_mask_color075()) { m.set_color_3(CharacterAppearance::hash_mask_color075(), app.mask_color075()); }
    if AssetTable::has_color(app.mask_color050()) && m.has_property(CharacterAppearance::hash_mask_color050()) { m.set_color_3(CharacterAppearance::hash_mask_color050(), app.mask_color075()); }
    if AssetTable::has_color(app.mask_color025()) && m.has_property(CharacterAppearance::hash_mask_color025()) { m.set_color_3(CharacterAppearance::hash_mask_color025(), app.mask_color025()); }
    if name.starts_with("MtHair") && AssetTable::has_color(app.hair_color()) && m.has_property(CharacterAppearance::hash_base_color()){
        m.set_color_3(CharacterAppearance::hash_base_color(), app.hair_color());
    }
    if (name.starts_with("MtSkin") || name.starts_with("MtuSkin")) && AssetTable::has_color(app.skin_color()) && m.has_property(CharacterAppearance::hash_base_color()){
        m.set_color_3(CharacterAppearance::hash_base_color(), app.skin_color());
    }
}
pub fn apply_colors_2(this: CharacterAppearance, go: GameObject) {
    if go.is_null() { return; }
    let data = UnitAssetMenuData::get();
    let data2 =
        if data.is_preview { Some(data.preview.preview_data.clone()) }
        else {
            let hash = unity::field_get_value_at_offset::<i32>(this, 0xd4);
            UnitAssetMenuData::get_by_person_data(hash, false)
                .and_then(|p| p.profile.get(p.profile_index(false) as usize).cloned())
        };
    if let Some(data2) = data2.as_ref() {
        let flag = data2.flag;
        this.m_instanced_materials().iter().for_each(|m| {
            let material_name = m.get_name().to_rust_string();
            if material_name.starts_with("MtHair2") || material_name.starts_with("MtOdd") {
                if let Some(color) = try_get_preview_color(14, data, data2) {
                    if m.has_property(CharacterAppearance::hash_base_color()) {
                        m.set_color_3(CharacterAppearance::hash_base_color(), color);
                    }
                }
            }
            else if material_name.starts_with("MtSkin") {
                if try_get_preview_color(2, data, data2).is_some() {
                    if AssetTable::has_color(this.skin_color()) { m.set_float("_Makeup", 0.0); }
                }
            }
            else if material_name.starts_with("MtEye") && !material_name.contains("Eye2"){
                for j in 0..6 {
                    if let Some(color) = try_get_preview_color(j+8, data, data2) {
                        m.set_color_2(UnitAssetData::EYE_COLOR[j as usize], color);
                    }
                }
            }
        });
        head_acc(go, flag & 64 != 0);
        hair_acc(go, flag & 16 != 0);
    }
}
fn try_get_preview_color(color_index: i32, data: &UnitAssetMenuData, d: &PlayerOutfitData) -> Option<Color> {
    let mut rgb = None;
    let j = color_index as usize;
    if data.is_preview && data.preview.color_preview[j * 4 + 3] == 1 {
        rgb = Some([data.preview.color_preview[j*4], data.preview.color_preview[j * 2], data.preview.color_preview[j*4 + 3]]);
    }
    else { if d.colors[j].values[3] != 0 {
        rgb = Some([d.colors[j].values[0], d.colors[j].values[1], d.colors[j].values[2]]); }
    }
    if let Some(rgb) = rgb.filter(|r| r[0] > 0 || r[1] > 0 || r[2] > 0) {
        let (r, g, b) = (rgb[0] as f32 / 255.0, rgb[1] as f32 / 255.0, rgb[2] as f32 / 255.0);
        return Some(Color { r, g, b, a: 1.0 });
    }
    None
}
