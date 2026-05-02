use cobapi::{Event, SystemEvent};
use engage::{gamemessage::GameMessage, proc::ProcInst, sequence::mainsequence::MainSequence};
use skyline::patching::Patch;
pub use outfit_core::UnitAssetMenuData;
#[allow(static_mut_refs)] pub mod enums;
#[allow(static_mut_refs)] pub mod assets;

// Required to get `event_install` function to fully compile?
extern "C" fn dvc_check_warning(event: &Event<SystemEvent>) {
    if let Event::Args(ev) = event {
        match ev {
            SystemEvent::ProcInstBind {proc, parent: _} => {
                let hash = proc.borrow().hashcode;
                match hash {
                    engage::proc::TITLE_LOOP_SEQUENCE => {
                        if let Some(main_sequence) = MainSequence::get_instance() {
                            if main_sequence.pad == 1 {
                                main_sequence.pad += 1;
                                GameMessage::create_key_wait(
                                    main_sequence,
                                    "Outfit plugin will be ignored for this session.\nDVC's version of the Outfit Plugin will be used."
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => { }
        }
    }
}
extern "C" fn event_install(event: &Event<SystemEvent>) {
    if MainSequence::get_instance().is_some_and(|v| v.pad == 0) {
        if let Event::Args(ev) = event {
            match ev {
                SystemEvent::ProcInstBind { proc, parent: _ } => {
                    let hash = proc.borrow().hashcode;
                    match hash {
                        engage::proc::TITLE_LOOP_SEQUENCE => {
                            println!("[Outfit Plugin {}]", outfit_core::VERSION);
                            if !UnitAssetMenuData::get().init {
                                outfit_core::install_outfit_plugin(false);
                                skyline::install_hooks!(
                                    assets::asset_table_setup_person_outfit,
                                    assets::asset_table_result_setup_hook_outfit,
                                    assets::transform::change_dragon2,
                                    assets::asset_table_result_god_setup_outfit,
                                    assets::transform::transformation_chain_atk,
                                    assets::create_break_effect_hook,
                                );
                            } else { outfit_core::reset_faces(true); }
                        }
                        engage::proc::UNIT_SELECT_SUB_MENU => { menu_item_add(&mut proc.borrow_mut()); }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}
fn menu_item_add(proc: &mut ProcInst) { outfit_core::add_sub_unit_menu_item(proc); }
#[skyline::main(name = "outfits")]
pub fn main() {
    cobapi::register_system_event_handler(event_install);
    cobapi::register_system_event_handler(dvc_check_warning);
    Patch::in_text(0x2517830).bytes(&[0xc0, 0x02, 0x80, 0x52]).unwrap();   // GameUserData Version 21
    Patch::in_text(0x1bb5f88).bytes(&[0x15, 0x00, 0x80, 0x52]).unwrap();    // Bypass the default variable in generating cutscene characters.
    std::panic::set_hook(Box::new(|info| {
        let location = info.location().unwrap();
        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => {
                match info.payload().downcast_ref::<String>() {
                    Some(s) => &s[..],
                    None => "Box<Any>",
                }
            },
        };
        let err_msg = format!(
            "Outfit Plugin has panicked at '{}' with the following message:\n{}\0",
            location,
            msg
        );
        skyline::error::show_error(
            6,
            "Outfit Plugin has panicked! Please open the details and send a screenshot to triabolical, then close the game.\n\0",
            err_msg.as_str(),
        );
    }));
}