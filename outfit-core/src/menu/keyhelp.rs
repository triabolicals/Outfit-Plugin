use engage::mess::Mess;
use engage::titlebar::{KeyHelpButton, KeyHelpButton::*, TitleBar};
use engage::keyhelp::*;
use unity::system::Il2CppString;
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
            TitleBar::open_header("Outfit Menu (Unit Info)", VERSION, "");
            if let Some(key) = TitleBar::get_instance().current_title.as_ref().map(|v| &v.key_help) {
                key.set_text_by_key(Plus, "Hide".into());
                key.set_text_by_key(LStick, "Move Unit".into());
                key.set_text_by_key(RStick, format!("Move Unit Z / {}", Mess::get("MID_KEYHELP_EDIT_TURN"), ).into());
            }
        }
        OutfitMenuKind::Photo => {
            KeyHelp::set_visible(false);
            TitleBar::open_header("Outfit Menu (Photo Mode)", VERSION, "");
            if let Some(key) = TitleBar::get_instance().current_title.as_ref().map(|v| &v.key_help) {
                key.set_text_by_key(Plus, "Hide".into());
                key.set_text_by_key(LStick, Mess::get("MID_KEYHELP_MENU_CAMERA_OPERATION"));
                key.set_text_by_key(RStick, format!("Camera Z / {}",  Mess::get("MID_KEYHELP_EDIT_TURN")).into());
                key.set_text_by_key(LR, "Roll".into());
                key.set_text_by_key(ZlZr, "Tilt".into());
            }
        }
        _ => {}
    }
}
pub fn add_key_help<'a>(key_help_button: KeyHelpButton, text: impl Into<&'a Il2CppString>) {
    if let Some(key) = TitleBar::get_instance().current_title.as_ref().map(|v| &v.key_help) {
        key.set_text_by_key(key_help_button, text.into());
    }
}
pub fn disable_key_help(key_help_button: KeyHelpButton){
    if let Some(key) = TitleBar::get_instance().current_title.as_ref().map(|v| &v.key_help) {
        key.disable(key_help_button);
    }
}