use engage::{
    combat::{CharacterAppearance, CharacterFactoryAsync},
    gamedata::{Gamedata, GodData, PersonData},
    gamedata::assettable::AssetTableResult,
    menu::{menu_item::BasicMenuItem, BasicMenuFields, BasicMenuResult},
    sequence::Bindable, unit::UnitPool, sequence::photograph::*,
    unityengine::{Camera, UnityComponent, UnityObject, UnityTransform}
};
use unity::{prelude::*, system::action::{SystemDelegate, Action}};
use crate::{print_asset_table_result, CustomAssetMenu, MenuMode, UnitAssetMenuData};

#[unity::class("App", "PhotographEditDisposMenu")]
pub struct PhotographEditDisposMenu {
    pub menu: BasicMenuFields<BasicMenuItem>,
    pub dispos_manager: &'static mut PhotographDisposManager, // Offset 0xC8, Attr: 1
}
impl Bindable for PhotographEditDisposMenu {}

pub fn photograph_edit_dispos_menu_minus(this: &PhotographEditDisposMenu, _optional_method: OptionalMethod) -> BasicMenuResult {
    if PersonData::get(this.dispos_manager.current_dispos_info.current_character_id).is_some_and(|v| v.parent.index > 0){
        if let Some(unit) = UnitPool::get_from_pid(this.dispos_manager.current_dispos_info.current_character_id, false).filter(|v| v.person.parent.index > 0){
            UnitAssetMenuData::get().mode = MenuMode::PhotoGraph;
            UnitAssetMenuData::set_unit(unit);
            CustomAssetMenu::photo_graph_bind(this);
            BasicMenuResult::se_cursor().with_close_this(true)
        }
        else { BasicMenuResult::new() }
    }
    else if let Some(god) = GodData::get(this.dispos_manager.current_dispos_info.current_character_id){
        UnitAssetMenuData::get().mode = MenuMode::PhotoGraph;
        UnitAssetMenuData::set_god(god);
        CustomAssetMenu::photo_graph_bind(this);
        BasicMenuResult::se_cursor().with_close_this(true)
    }
    else { BasicMenuResult::new() }
}

pub fn update_character(dispos_info: &'static mut PhotographDisposInfo, result: &AssetTableResult) {
    let obj = PhotographDisposInfo77::instantiate();
    let locator = dispos_info.m_locator.get_transform();
    print_asset_table_result(result, 2);
    obj.character_cmp = CharacterFactoryAsync::create_for_talk(CharacterAppearance::create_from_result(result, 1), locator, true);
    obj.character_id = dispos_info.current_character_id;
    obj.body_acc = dispos_info.body_acc_data;
    obj.face_acc = dispos_info.face_acc_data;
    obj.this = dispos_info;
    obj.this.m_is_loading_character = true;
    let start = Action::instantiate().unwrap();
    let method = obj.klass.get_methods()[1];
    start.ctor(Some(obj), method);
    start.method_ptr = set_up as _;
    obj.character_cmp.call_on_setup_done(start);
}
fn set_up(this: &'static mut PhotographDisposInfo77, _optional_method: OptionalMethod) {
    if let Some(char) = this.this.m_character_cmp.as_ref() {
        if let Some(go) = char.get_game_object().filter(|v| !v.is_null()) {
            go.set_active2(false);
            go.destroy();
        }
    }
    this.this.m_character_cmp = Some(this.character_cmp);
    this.this.m_is_loading_character = false;
    if let Some((char, camera_trans)) = this.this.m_character_cmp.as_ref().zip(Camera::get_main().map(|v| v.get_transform())){
        let menu_data = UnitAssetMenuData::get();

        char.get_transform().set_local_rotation(menu_data.control.current_character.rotation);
        char.set_is_visible(true);
        let joint = char.get_joint();
        if let Some(head_go) = joint.get_c_head_loc().and_then(|t| t.get_game_object()){
            if let Some(char_go) = char.get_game_object(){
                char_go.get_components_in_children::<LookAtIK>(true).iter_mut().for_each(|v|{
                    if let Some(look_go) = v.get_transform().get_parent().and_then(|t| t.get_game_object()).filter(|v| !v.is_null()){
                        if look_go.equals(head_go) {
                            this.this.look_at_ik_eye = v;
                            this.this.look_at_ik_eye.solver.target = camera_trans;
                            this.this.look_at_ik_eye.solver.eyes_weight = 0.5;
                        }
                        else if look_go.equals(char_go){
                            this.this.look_at_ik_body = v;
                            let look_at_trans = this.this.m_look_target.get_transform();
                            this.this.look_at_ik_body.solver.target = look_at_trans;
                            this.this.look_at_ik_body.solver.body_weight = 0.3;
                            this.this.look_at_ik_body.solver.head_weight = 0.5;
                        }
                    }
                });
            }
        }
    }
    this.this.set_up_pause();
    this.this.setup_weapon();
    if let Some(char) = this.this.m_character_cmp.as_ref() {
        let menu_data = UnitAssetMenuData::get();
        char.play_facial(crate::FACIAL_STATES[menu_data.facial].into());
    }
}

