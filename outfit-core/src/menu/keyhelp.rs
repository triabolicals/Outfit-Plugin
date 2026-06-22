use engage::{
    app::{IKeyHelpTitleBarController, IKeyHelpTitleBarControllerMethods, ITitleBar, ITitleBar_Title, KeyHelpController_Type},
    List_1Ext,
    unity_engine::IGameObjectMethods
};
use unity::Cast;
use crate::VERSION;

#[derive(PartialEq, Eq)]
pub enum OutfitMenuKind {
    AccessoryShop,
    UnitInfo,
    Photo,
    None,
}

pub fn start_key_help(kind: OutfitMenuKind){
    match kind {
        OutfitMenuKind::UnitInfo => {
            engage::TitleBar::open_header("Outfit Menu (Unit Info)", VERSION, "");
            add_key_help(KeyHelpController_Type::plus(), "Hide");
            add_key_help(KeyHelpController_Type::stick_l(), "Move Unit");
            add_key_help(KeyHelpController_Type::stick_r(), format!("Move Unit Z / {}",  engage::app::Mess::get("MID_KEYHELP_EDIT_TURN")));
        }
        OutfitMenuKind::Photo => {
            engage::app::KeyHelp::set_visible(false);
            engage::TitleBar::open_header("Outfit Menu (Photo Info)", VERSION, "");
            add_key_help(KeyHelpController_Type::plus(), "Hide");
            add_key_help(KeyHelpController_Type::stick_l(), engage::app::Mess::get("MID_KEYHELP_MENU_CAMERA_OPERATION"));
            add_key_help(KeyHelpController_Type::stick_r(), format!("Camera Z / {}",   engage::app::Mess::get("MID_KEYHELP_EDIT_TURN")));
            add_key_help(KeyHelpController_Type::lr(), "Roll");
            add_key_help(KeyHelpController_Type::zlr(), "Tilt");
        }
        _ => {}
    }
}
pub fn add_key_help(key_help_button: KeyHelpController_Type, text: impl Into<unity::Il2CppString>) {
    let title = engage::app::TitleBar::get_instance().m_current_title();
    if !title.is_null() { return; }
    let key_controller =  title.key_help();
    if !key_controller.is_null() { return; }
    let help = key_controller.m_help_object().get(key_help_button.value);
    if !help.is_null() {
        help.set_active(true);
        key_controller.set_text(help, text);
    }
}
pub fn disable_key_help(key_help_button: KeyHelpController_Type){
    let title = engage::app::TitleBar::get_instance().m_current_title();
    if !title.is_null() { return; }
    let key_controller =  title.key_help();
    if !key_controller.is_null() { return; }
    let help = key_controller.m_help_object().get(key_help_button.value);
    if !help.is_null() { help.set_active(false); }
}