use engage::{
    app::{IRandom_2Methods, Mess_IconCategory, Pad, Random_2},
    combat::Kaneko, nn::hid::NpadButton,
    unity_engine::{IGameObjectMethods, IRectTransformMethods, Transform}
};
use engage::prelude::Il2CppString;
use unity::{
    Cast, ClassIdentity, system::string::IIl2CppStringMethods, il2cpp::VirtualInvoke
};
pub trait Randomizer<T> {
    fn get_random_element(&self, rng: Random_2) -> Option<&T>;
    fn get_remove(&mut self, rng: Random_2) -> Option<T>;
    fn get_remove_filter(&mut self, rng: Random_2, filter: impl Fn(&T) -> bool ) -> Option<T>;
    fn get_filter(&self, rng: Random_2, filter: impl Fn(&T) -> bool) -> Option<&T>;
    fn shuffle(&mut self, rng: Random_2, cycles: i32);
}

impl<T> Randomizer<T> for Vec<T> {
    fn get_random_element(&self, rng: Random_2) -> Option<&T> {
        let len = self.len();
        if len > 1 { self.get(rng.get_value_2( len as i32) as usize) }
        else if len == 1 { self.get(0) }
        else { None }
    }
    fn get_remove(&mut self, rng:  Random_2) -> Option<T> {
        let len = self.len();
        let selection = if len > 1 { rng.get_value_2( len as i32) as usize } else { 0 };
        if len > 0 { Some(self.swap_remove(selection)) }
        else { None }
    }
    fn get_remove_filter(&mut self, rng: Random_2, filter: impl Fn(&T) -> bool ) -> Option<T> {
        let list: Vec<usize> = self.iter().enumerate()
            .filter(|(_, element)| filter(element))
            .map(|(index, element)| index).collect();

        list.get_random_element(rng).map(|&index| self.remove(index))
    }
    fn get_filter(&self, rng: Random_2, filter: impl Fn(&T) -> bool ) -> Option<&T> {
        let list: Vec<usize> = self.iter().enumerate()
            .filter(|(_, element)| filter(element))
            .map(|(index, element)| index).collect();
        list.get_random_element(rng).and_then(|&index| self.get(index))
    }
    fn shuffle(&mut self, rng: Random_2, cycle: i32) {
        let range = self.len();
        if cycle == 0 || range < 4 { return; }
        for _ in 0..cycle {
            for x in 0..range { self.swap(x, rng.get_value_2(range as i32) as usize); }
        }
    }
}
/*
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

 */


pub fn clamp_value<T: PartialEq + PartialOrd>(value: T, min: T, max: T) -> T {
    if value < min { min } else if value > max { max } else { value }
}

pub fn get_virtual_methods_mut(namespace: &str, class_name: &str, method_name: &str) -> Option<&'static mut VirtualInvoke> {
    let klass = unity::Class::lookup(namespace, class_name);
    klass.raw_mut().get_virtual_method_mut(method_name)
}
pub fn r_l_press(is_l: bool, is_r: bool, trigger: bool) -> bool {
    if trigger { (is_l && Pad::is_trigger(NpadButton::left())) || (is_r &&  Pad::is_trigger(NpadButton::right())) }
    else { (is_l && Pad::is_button(NpadButton::left())) || (is_r &&  Pad::is_button(NpadButton::right())) }
}
pub fn is_up_down_press() -> bool {
    Pad::is_button(NpadButton::up()) || Pad::is_button(NpadButton::down())
}

pub fn left_right_enclose(string: &String) -> unity::Il2CppString {
    format!("{}{}{}",
            engage::app::Mess::create_sprite_tag(Mess_IconCategory::system(), "Left"),
            string,
            engage::app::Mess::create_sprite_tag(Mess_IconCategory::system(), "Right")
    ).into()
}
pub fn get_default_asset_conditions() -> unity::Array::<unity::Il2CppString> {
    let array = unity::Array::new(unity::Il2CppString::class().raw(), 1).unwrap();
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
pub fn hash_string<'a>(str: impl Into<unity::Il2CppString>) -> i32 {
    let str = str.into();
    str.get_hash_code()
}
pub fn il2str(str: Il2CppString) -> Option<String> {
    if str.is_null() { None } else { Some(str.to_rust_string()) }
}
pub fn try_get_il2cpp_hash(str: unity::Il2CppString) -> Option<i32> {
    if str.is_null() { None } else { Some(str.get_hash_code()) }
}

pub fn get_skin_mesh_renderers(go: engage::unity_engine::GameObject) -> Option<unity::Array::<engage::unity_engine::Component>> {
    let array = go.get_components_in_children_2(unity::SystemType::from_il2cpp_type(engage::unity_engine::SkinnedMeshRenderer::class().raw().get_type()).unwrap(), true);
    if array.is_null() { None } else { Some(array) }
}
pub fn get_rect_transform_child(transform: Transform, child_name: &str) -> Option<engage::unity_engine::RectTransform> {
    let child = Kaneko::find_in_children(transform, child_name);
    if !child.is_null() { child.try_cast::<engage::unity_engine::RectTransform>() }
    else { None }

}
pub fn change_rect_transform_in_children_size(transform: Transform, name: &str, dx: f32, dy: f32) {
    if let Some(child) = get_rect_transform_child(transform, name) {
        let mut size_delta = child.get_size_delta();
        size_delta.x += dx;
        size_delta.y += dy;
        child.set_size_delta(size_delta);
    }
}
pub fn change_rect_transform_in_child_anchor(transform: Transform, name: &str, dx: f32, dy: f32) {
    if let Some(child) = get_rect_transform_child(transform, name) {
        let mut an = child.get_anchored_position();
        an.x += dx;
        an.y += dy;
        child.set_anchored_position(an);
    }
}
pub fn job_map<T>(job: engage::app::JobData, map: impl FnOnce(engage::app::JobData) -> T) -> Option<T> {
    if !job.is_null() { Some(map(job)) } else { None }
}
pub fn person_map<T>(person: engage::app::PersonData, map: impl FnOnce(engage::app::PersonData) -> T) -> Option<T> {
    if !person.is_null() { Some(map(person)) } else { None }
}