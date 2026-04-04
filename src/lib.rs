use cobapi::{Event, SystemEvent};
use engage::gamemessage::GameMessage;
use skyline::patching::Patch;
pub use outfit_core::UnitAssetMenuData;
use engage::sequence::mainsequence::MainSequence;

#[allow(static_mut_refs)] pub mod enums;
#[allow(static_mut_refs)] pub mod assets;

extern "C" fn event_install(event: &Event<SystemEvent>) {
    if let Event::Args(ev) = event {
        match ev {
            SystemEvent::ProcInstBind {proc, parent: _} => {
                match proc.borrow().hashcode {
                    engage::proc::TITLE_LOOP_SEQUENCE => {
                        if let Some(main_sequence) = MainSequence::get_instance() {
                            if main_sequence.pad != 1 {
                                if !UnitAssetMenuData::get().init {
                                    outfit_core::install_outfit_plugin(false);
                                    skyline::install_hooks!(
                                    assets::asset_table_setup_person_outfit,
                                    assets::asset_table_result_setup_hook_outfit,
                                    assets::transform::change_dragon2,
                                    assets::asset_table_result_god_setup_outfit,
                                    assets::transform::transformation_chain_atk,
                                );
                                }
                            }
                            else {
                                UnitAssetMenuData::get().init = true;
                                GameMessage::create_key_wait(
                                    main_sequence,
                                    "Outfit plugin will not be installed.\nDraconic Vibe Crystal's version will be used instead.\nRemove the Outfit plugin to improve performance.");
                            }
                        }
                    }
                    _ => {}
                }
            }
            _ => {},
        }
    } 
    else { }
}
#[skyline::main(name = "outfits")]
pub fn main() {
    cobapi::register_system_event_handler(event_install);
    Patch::in_text(0x2517830).bytes(&[0xc0, 0x02, 0x80, 0x52]).unwrap();   // GameUserData Version 21
    Patch::in_text(0x1bb5f88).bytes(&[0x15, 0x00, 0x80, 0x52]).unwrap();    // Bypass the default variable in generating cutscene characters.
    
    // Patch::in_text(0x0228151c).bytes(&[0x8A, 0x0C, 0x80, 0x52]).unwrap();
    // Patch::in_text(0x02281fb8).bytes(&[0x88, 0x0C, 0x80, 0x52]).unwrap();

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