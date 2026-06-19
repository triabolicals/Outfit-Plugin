use engage_il2cpp::{
    app::{AssetTable_Result, IPhotographDisposInfo},
    system::Object,
    unity_engine::IGameObjectMethods,
    app::{BasicMenu_Result, IPhotographDisposInfoMethods, IPhotographDisposManager, IPhotographEditDisposMenu, IProcInstMethods, ISingletonProcInst_1Methods, IStructBase, IStructData_1Methods, PhotographEditDisposMenu},
    combat::{CharacterFactoryAsync, ICharacterJointMethods, ICharacterMethods},
    unity_engine::IComponentMethods,
    system::{Action, object::*},
};
use unity::{prelude::*};
use unity2::{Cast, FromIlInstance};
use crate::{CustomAssetMenu, MenuMode, UnitAssetMenuData};

pub fn get_photosequence() -> Option<engage_il2cpp::app::PhotographSequence> {
    let p = engage_il2cpp::app::PhotographTopSequence::get_instance();
    if !p.is_null() {
        p.get_child().try_cast()
    }
    else { None }
}

pub fn photograph_edit_dispos_menu_minus(this: PhotographEditDisposMenu, _optional_method: unity2::OptionalMethod) -> BasicMenu_Result {
    let current_character_id = this.m_dispos_manager().m_current_dispos_info().get_current_character_id();
    if current_character_id.is_null() { BasicMenu_Result::pass() }
    else {
        let person = engage_il2cpp::app::PersonData::get(current_character_id);
        if !person.is_null() {
            if person.index() > 0 {
                let unit = engage_il2cpp::app::UnitPool::get_from_person(person, false);
                if !unit.is_null() {
                    UnitAssetMenuData::get().mode = MenuMode::PhotoGraph;
                    UnitAssetMenuData::set_unit(unit);
                    CustomAssetMenu::create_photo_graph_bind(this);
                    return BasicMenu_Result::close_decide();
                }
            }
        }
        let god = engage_il2cpp::app::GodData::get(current_character_id);
        if !god.is_null() {
            UnitAssetMenuData::get().mode = MenuMode::PhotoGraph;
            UnitAssetMenuData::set_god(god);
            CustomAssetMenu::create_photo_graph_bind(this);
            return BasicMenu_Result::close_decide();
        }
        BasicMenu_Result::pass()
    }
}
#[unity2::inject(namespace = "App", name = "CreatePhotographCharacter", parent=Object)]
pub struct CreatePhotographCharacter {
    pub dispos_info: engage_il2cpp::app::PhotographDisposInfo,
    pub character: engage_il2cpp::combat::Character,
    pub character_id: unity2::Il2CppString,
    pub body_acc: engage_il2cpp::app::AccessoryData,
    pub face_acc: engage_il2cpp::app::AccessoryData,
}

pub fn update_character(dispos_info: engage_il2cpp::app::PhotographDisposInfo, result: AssetTable_Result) {
    let obj = CreatePhotographCharacter::instantiate().unwrap();
    let locator = dispos_info.m_locator().get_transform();
    obj.set_dispos_info(dispos_info);
    obj.set_character(CharacterFactoryAsync::create_for_talk(engage_il2cpp::combat::CharacterAppearance::create_from_result(result, 1), locator, true));
    obj.set_character_id(dispos_info.get_current_character_id());
    obj.set_body_acc(dispos_info.get_body_acc_data());
    obj.set_face_acc(dispos_info.get_face_acc_data());
    dispos_info.set_m_is_loading_character(true);
    let start = Action::new(obj.into(), set_up_method_info().into());
    obj.character().call_on_setup_done(start);
}
#[unity2::callback]
fn set_up(this: CreatePhotographCharacter, _: unity2::OptionalMethod) {
    let current_character = this.dispos_info().m_character_cmp();
    if !current_character.is_null() {
        let go = current_character.get_game_object();
        if !go.is_null() {
            go.set_active(false);
            engage_il2cpp::unity_engine::Object_2::destroy_2(go);
        }
    }
    let dispos_info = this.dispos_info();
    let new_character = this.character();
    if !new_character.is_null() {
        dispos_info.set_m_character_cmp(new_character);
        dispos_info.set_m_is_loading_character(false);
        let camera = engage_il2cpp::unity_engine::Camera::get_main();
        if !camera.is_null() {
            let camera_transform = camera.get_transform();
            new_character.set_is_visible(true);
            let character_joint = new_character.get_joint();
            let head_loc = character_joint.get_c_head_loc();
            let character_go = new_character.get_game_object();
            // character_go.get_components_in_children_3::<LookAtIK>(true).iter().for_each(|l| {});
        }
    }
    dispos_info.set_up_pause();
    dispos_info.setup_weapon();
}
/*
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
        char.play_facial(crate::FACIAL_STATES[menu_data.facial].0.into());
    }
}
 */
