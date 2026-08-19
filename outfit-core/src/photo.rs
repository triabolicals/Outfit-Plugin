use engage::{
    app::{AssetTable_Result, IPhotographDisposInfo},
    system::Object,
    app::{BasicMenu_Result, IPhotographDisposInfoMethods, IPhotographDisposManager, IPhotographEditDisposMenu, IProcInstMethods, ISingletonProcInst_1Methods, IStructBase, IStructData_1Methods, PhotographEditDisposMenu},
    combat::{CharacterFactoryAsync, ICharacterJointMethods, ICharacterMethods},
    unity_engine::{IComponentMethods, IGameObjectMethods, ITransformMethods},
    system::{Action, object::*},
    root_motion::final_ik::{IIKSolverLookAt, ILookAtIK},
};
use unity::{Cast, FromIlInstance};
use crate::{CustomAssetMenu, MenuMode, UnitAssetMenuData};

pub fn get_photosequence() -> Option<engage::app::PhotographSequence> {
    let p = engage::app::PhotographTopSequence::get_instance();
    if !p.is_null() { p.get_child().try_cast() } else { None }
}
pub fn photograph_edit_dispos_menu_minus(this: PhotographEditDisposMenu, _optional_method: unity::OptionalMethod) -> BasicMenu_Result {
    let current_character_id = this.m_dispos_manager().m_current_dispos_info().get_current_character_id();
    if current_character_id.is_null() { BasicMenu_Result::pass() }
    else {
        let person = engage::app::PersonData::get(current_character_id);
        if !person.is_null() {
            if person.index() > 0 {
                let unit = engage::app::UnitPool::get_from_person(person, false);
                if !unit.is_null() {
                    UnitAssetMenuData::get().mode = MenuMode::PhotoGraph;
                    UnitAssetMenuData::set_unit(unit);
                    CustomAssetMenu::create_photo_graph_bind(this);
                    return BasicMenu_Result::close_decide();
                }
            }
        }
        let god = engage::app::GodData::get(current_character_id);
        if !god.is_null() {
            UnitAssetMenuData::get().mode = MenuMode::PhotoGraph;
            UnitAssetMenuData::set_god(god);
            CustomAssetMenu::create_photo_graph_bind(this);
            return BasicMenu_Result::close_decide();
        }
        BasicMenu_Result::pass()
    }
}
#[unity::inject(namespace = "App", name = "CreatePhotographCharacter", parent=Object)]
pub struct CreatePhotographCharacter {
    pub dispos_info: engage::app::PhotographDisposInfo,
    pub character: engage::combat::Character,
    pub character_id: unity::Il2CppString,
    pub body_acc: engage::app::AccessoryData,
    pub face_acc: engage::app::AccessoryData,
}

pub fn update_character(dispos_info: engage::app::PhotographDisposInfo, result: AssetTable_Result) {
    let obj = CreatePhotographCharacter::instantiate().unwrap();
    let locator = dispos_info.m_locator().get_transform();
    obj.set_dispos_info(dispos_info);
    obj.set_character(CharacterFactoryAsync::create_for_talk(engage::combat::CharacterAppearance::create_from_result(result, 1), locator, true));
    obj.set_character_id(dispos_info.get_current_character_id());
    obj.set_body_acc(dispos_info.get_body_acc_data());
    obj.set_face_acc(dispos_info.get_face_acc_data());
    dispos_info.set_m_is_loading_character(true);
    let start = Action::new(obj.into(), set_up_method_info().into());
    obj.character().call_on_setup_done(start);
}
#[unity::callback]
fn set_up(this: CreatePhotographCharacter, _: unity::OptionalMethod) {
    let current_character = this.dispos_info().m_character_cmp();
    if !current_character.is_null() {
        let go = current_character.get_game_object();
        if !go.is_null() {
            go.set_active(false);
            engage::unity_engine::Object_2::destroy_2(go);
        }
    }
    let dispos_info = this.dispos_info();
    let new_character = this.character();
    if !new_character.is_null() {
        dispos_info.set_m_character_cmp(new_character);
        dispos_info.set_m_is_loading_character(false);
        let camera = engage::unity_engine::Camera::get_main();
        if !camera.is_null() {
            let camera_transform = camera.get_transform();
            new_character.set_is_visible(true);
            let character_joint = new_character.get_joint();
            if let Some(head_go) = to_option(character_joint.get_c_head_loc()).and_then(|t| to_option(t.get_game_object())) {
                let character_go = new_character.get_game_object();
                character_go.get_components_in_children_5::<engage::root_motion::final_ik::LookAtIK>().iter()
                    .filter(|c| !c.is_null())
                    .for_each(|c| {
                        if let Some(go) = to_option(c.get_transform())
                            .and_then(|t| to_option(t.get_parent()))
                            .and_then(|t| to_option(t.get_game_object()))
                        {
                            if engage::unity_engine::Object_2::op_equality(head_go, go) {
                                c.solver().set_target(camera_transform);
                                c.solver().set_eyes_weight(0.5);
                                this.dispos_info().set_m_look_at_ik_body(c);
                            }
                            else if engage::unity_engine::Object_2::op_equality(character_go, go){
                                c.solver().set_target(this.dispos_info().m_look_target().get_transform());
                                c.solver().set_body_weight(0.3);
                                c.solver().set_head_weight(0.5);
                                this.dispos_info().set_m_look_at_ik_body(c);
                            }
                        }

                    });
            }
        }
    }
    dispos_info.set_up_pause();
    dispos_info.setup_weapon();
}
pub fn to_option<T: Cast>(v: T) -> Option<T> { if v.is_null() { None } else { Some(v) } }