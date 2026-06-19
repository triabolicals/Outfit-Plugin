use engage::{gamedata::assettable::AssetTableResult, mess::Mess, random::Random, util::get_instance};
use engage_il2cpp::app::{IRandom_2Methods, Mess_IconCategory, Random_2};
use engage_il2cpp::unity_engine::IGameObjectMethods;
use unity::{il2cpp::class::VirtualInvoke, prelude::*};
use unity2::ClassIdentity;
use unity2::system::string::IIl2CppStringMethods;
use crate::assets::new_asset_table_accessory;
pub trait Randomizer<T> {
    fn get_random_element(&self, rng: Random_2) -> Option<&T>;
    fn get_remove(&mut self, rng: Random_2) -> Option<T>;
}

impl<T> Randomizer<T> for Vec<T> {
    fn get_random_element(&self, rng: Random_2) -> Option<&T> {
        let len = self.len();
        if len > 1 { self.get(rng.get_value_2( len as i32) as usize) }
        else { None }
    }
    fn get_remove(&mut self, rng:  Random_2) -> Option<T> {
        let len = self.len();
        let selection = if len > 1 { rng.get_value_2( len as i32) as usize } else { 0 };
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
pub fn r_l_press(is_l: bool, is_r: bool, trigger: bool) -> bool {
    let pad = get_instance::<engage::pad::Pad>();
    if trigger && ( pad.old_buttons.right() || pad.old_buttons.left() ) { false }
    else { is_l == pad.npad_state.buttons.left() && is_r == pad.npad_state.buttons.right() }
}
pub fn is_up_down_press() -> bool {
    let pad = get_instance::<engage::pad::Pad>();
    pad.old_buttons.up() || pad.old_buttons.down() || pad.npad_state.buttons.up() || pad.npad_state.buttons.down()
}

pub fn left_right_enclose(string: &String) -> unity2::Il2CppString {
    format!("{}{}{}",
            engage_il2cpp::app::Mess::create_sprite_tag(Mess_IconCategory::system(), "Left"),
            string,
            engage_il2cpp::app::Mess::create_sprite_tag(Mess_IconCategory::system(), "Left")
    ).into()
}
pub fn get_default_asset_conditions() -> unity2::Array::<unity2::Il2CppString> {
    let array = unity2::Array::new(unity2::Il2CppString::class().raw(), 1).unwrap();
    array.set(0, "".into());
    array
}
pub fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}
pub fn hash_string<'a>(str: impl Into<&'a Il2CppString>) -> i32 {
    let str = str.into();
    str.get_hash_code()
}
pub fn il2str(str: unity2::Il2CppString) -> Option<String> {
    if str.is_null() { None } else { Some(str.to_rust_string()) }
}
pub fn try_get_il2cpp_hash(str: unity2::Il2CppString) -> Option<i32> {
    if str.is_null() { None } else { Some(str.get_hash_code()) }
}

pub fn get_skin_mesh_renderers(go: engage_il2cpp::unity_engine::GameObject) -> Option<unity2::Array::<engage_il2cpp::unity_engine::Component>> {
    let array = go.get_components_in_children_2(unity2::SystemType::from_il2cpp_type(engage_il2cpp::unity_engine::SkinnedMeshRenderer::class().raw().get_type()).unwrap(), true);
    if array.is_null() { None } else { Some(array) }
}
