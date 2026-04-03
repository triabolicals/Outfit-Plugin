use engage::gamedata::assettable::AssetTableResult;
use engage::mess::Mess;
use engage::random::Random;
use engage::util::get_instance;
use unity::il2cpp::class::VirtualInvoke;
use unity::macro_context::Il2CppClass;
use unity::prelude::Il2CppString;
use crate::assets::new_asset_table_accessory;
pub trait Randomizer<T> {
    fn get_random_element(&self, rng: &Random) -> Option<&T>;
    fn get_remove(&mut self, rng: &Random) -> Option<T>;
}

impl<T> Randomizer<T> for Vec<T> {
    fn get_random_element(&self, rng: &Random) -> Option<&T> {
        let len = self.len();
        if len > 1 { self.get(rng.get_value( len as i32) as usize) }
        else { None }
    }
    fn get_remove(&mut self, rng: &Random) -> Option<T> {
        let len = self.len();
        let selection = if len > 1 { rng.get_value( len as i32) as usize } else { 0 };
        if len > 0 { Some(self.swap_remove(selection)) }
        else { None }
    }
}

pub fn print_asset_table_result(result: &AssetTableResult, mode: i32) {
    if let Some(pid) = result.pid.as_ref() { println!("Asset Table Result PID: {} [Mode: {}]", Mess::get_name(pid.to_string().as_str()), mode); }
    else { println!("Asset Table Result Mode: {}", mode); }
    if let Some(jid) = result.jid.as_ref() { println!("JID: {}", Mess::get_name(jid.to_string().as_str())); }
    if let Some(ride) = result.ride_model.as_ref() { println!("Ride Model: {}", ride); }
    if let Some(ride_dress) = result.ride_dress_model.as_ref() { println!("Ride Dress Model: {}", ride_dress); }
    if !result.dress_model.is_null() { println!("Dress Model: {}", result.dress_model); }
    if !result.body_model.is_null() { println!("Body Model: {}", result.body_model); }
    if !result.head_model.is_null() { println!("Head Model: {}", result.head_model); }
    if !result.hair_model.is_null() { println!("Hair Model: {}", result.hair_model); }
    result.body_anims.iter().enumerate().for_each(|(a, i)|{ println!("Body Anim #{}: {}", a, i); });
    if let Some(body) = result.body_anim.as_ref() { println!("Body Anim: {}", body); }
    if let Some(aoc) = result.info_anims.as_ref() { println!("Info: {}", aoc); }
    if let Some(aoc) = result.talk_anims.as_ref() { println!("Talk: {}", aoc); }
    if let Some(aoc) = result.demo_anims.as_ref() { println!("Demo: {}", aoc); }
    if let Some(aoc) = result.hub_anims.as_ref() { println!("Hub: {}", aoc); }
    if let Some(voice) = result.sound.voice.as_ref() { println!("Voice: {}", voice); }
}


pub fn clamp_value<T: PartialEq + PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min { min } else if value > max { max } else { value }
}

pub fn get_nested_virtual_methods_mut(namespace: &str, class_name: &str, nested_class: &str, method_name: &str) -> Option<&'static mut VirtualInvoke> {
    if let Some(cc) = Il2CppClass::from_name(namespace, class_name).unwrap().get_nested_types().iter()
        .find(|x| x.get_name() == nested_class) {
        let menu_mut = Il2CppClass::from_il2cpptype(cc.get_type()).unwrap();
        menu_mut.get_virtual_method_mut(method_name)
    }
    else { None }
}
pub fn apply_hair(hair: &String, result: &mut AssetTableResult) {
    if hair.contains("spine") {
        let accessory = new_asset_table_accessory(hair.to_string().as_str(), "c_spine1_jnt");
        result.commit_accessory(&accessory);
        result.hair_model = "uHair_null".into();
    }
    else {
        let accessory = new_asset_table_accessory("null", "c_spine1_jnt");
        result.commit_accessory(&accessory);
        result.hair_model = hair.into();
    }
    result.replace(2);
}
pub fn r_l_press(is_l: bool, is_r: bool, trigger: bool) -> bool {
    let pad = get_instance::<engage::pad::Pad>();
    if trigger && ( pad.old_buttons.right() || pad.old_buttons.left() ) { false }
    else { is_l == pad.npad_state.buttons.left() && is_r == pad.npad_state.buttons.right() }
}
pub fn is_up_down_press() -> bool {
    let pad = get_instance::<engage::pad::Pad>();
    pad.old_buttons.up() || pad.old_buttons.down() || pad.npad_state.buttons.up() || pad.npad_state.buttons.down()
}

pub fn left_right_enclose(string: &String) -> &'static Il2CppString {
    format!("{}{}{}", Mess::create_sprite_tag_str(2, "Left"), string, Mess::create_sprite_tag_str(2, "Right")).into()
}