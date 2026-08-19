use engage::{
    app::{AssetTable, AssetTable_Modes, AssetTable_Result, IAssetTable, IAssetTableMethods, IAssetTable_AccessoryMethods, IAssetTable_Result, IAssetTable_ResultMethods},
    system::collections::generic::IList_1,
    unity_engine::Color
};
use unity::{Cast, Il2CppString, OptionalMethod};
use crate::new_asset_table_accessory;

pub fn get_result_hash(result: AssetTable_Result, _optional_method: OptionalMethod) -> i32 {
    let mut hash = unsafe { AssetTable_Result::get_hash_code(result) };
    for x in 0..16 {
        let scale = get_result_scale_u16(result, x) as i32 / 10;
        hash = hash.wrapping_add( scale + (10 * x as i32));
    }
    for x in 0..8 {
        let color = get_result_color_i32(result, x);
        hash = hash.wrapping_add( color + (10 * (x as i32 + 16)));
    }
    hash
}

pub fn set_color_by_u8_slice(result: AssetTable_Result, idx: usize, v: [u8; 4]) {
    set_color_by_u8(result, idx, v[0], v[1], v[2]);
}
pub fn set_color_by_i32(result: AssetTable_Result, idx: usize, v: i32) {
    let mut rgb: [u8; 4] = [0; 4];
    for i in 0..3 {
        rgb[i] = ((v >> (i*8)) & 255) as u8;
    }
    set_color_by_u8_slice(result, idx, rgb);
}
pub fn set_color_by_u8(result: AssetTable_Result, idx: usize, r: u8, g: u8, b: u8) {
    let color = Color{
        r: r as f32 / 255.0,
        g: g as f32 / 255.0,
        b: b as f32 / 255.0,
        a: 1.0,
    };
    match idx {
        0 => result.set_hair_color(color),
        1 => result.set_grad_color(color),
        2 => result.set_skin_color(color),
        3 => result.set_toon_shadow_color(color),
        4 => result.set_mask_color100(color),
        5 => result.set_mask_color075(color),
        6 => result.set_mask_color050(color),
        7 => result.set_mask_color025(color),
        _ => {}
    }
}
pub fn get_result_color(result: AssetTable_Result, idx: usize) -> Color {
    match idx {
        1 => result.get_grad_color(),
        2 => result.get_skin_color(),
        3 => result.get_toon_shadow_color(),
        4 => result.get_mask_color100(),
        5 => result.get_mask_color075(),
        6 => result.get_mask_color050(),
        7 => result.get_mask_color025(),
        _ => result.get_hair_color(),
    }
}
pub fn get_result_color_i32(result: AssetTable_Result, idx: usize) -> i32 {
    let color = get_result_color(result, idx);
    let mut v = 0;
    v |= (color.r * 255.0) as i32;
    v |= ((color.g * 255.0) as i32) << 8;
    v |= ((color.b * 255.0) as i32) << 16;
    v
}
pub fn get_result_color_u8(result: AssetTable_Result, idx: usize) -> [u8; 3] {
    let color = get_result_color(result, idx);
    [(color.r * 255.0) as u8, (color.g * 255.0) as u8, (color.b * 255.0) as u8]
}
pub fn get_result_dress_body_model(result: AssetTable_Result, mode: i32) -> Il2CppString {
    if mode == 2 { result.get_dress_model() } else { result.get_body_model() }
}

pub fn set_result_dress_body_model(result: AssetTable_Result,  mode: i32, model: impl Into<Il2CppString>) {
    if mode == 2 { result.set_dress_model(model); }
    else { result.set_body_model(model); }
}

pub fn apply_result_hair(hair: &String, result: AssetTable_Result) {
    if hair.contains("spine") {
        let accessory = new_asset_table_accessory(hair.as_str(), "c_spine1_jnt");
        result.set_hair_model("uHair_null");
        result.commit_8(accessory);
    }
    else {
        let accessory = new_asset_table_accessory("null", "c_spine1_jnt");
        result.commit_8(accessory);
        result.set_hair_model(hair.as_str());
    }
    result.replace(AssetTable_Modes::combat());
}
pub fn set_result_scale_u16(result: AssetTable_Result, index: usize, scale: u16) {
    if scale > 0 && scale <= 1000 {
        let value = scale as f32 / 100.0;
        set_result_scale(result, index, value);
    }
}
pub fn set_result_scale(result: AssetTable_Result, index: usize, value: f32) {
    match index {
        0 => result.set_scale_all(value),
        1 => result.set_scale_head(value),
        2 => result.set_scale_neck(value),
        3 => result.set_scale_torso(value),
        4 => result.set_scale_shoulders(value),
        5 => result.set_scale_arms(value),
        6 => result.set_scale_hands(value),
        7 => result.set_scale_legs(value),
        8 => result.set_scale_feet(value),
        9 => result.set_volume_bust(value),
        10 => result.set_volume_abdomen(value),
        11 => result.set_volume_torso(value),
        12 => result.set_volume_base_arms(value),
        13 => result.set_volume_base_legs(value),
        14 => result.set_volume_scale_arms(value),
        15 => result.set_volume_scale_legs(value),
        16 => result.set_map_scale_all(value),
        17 => result.set_map_scale_head(value),
        18 => result.set_map_scale_wing(value),
        _ => {}
    }
}
pub fn get_result_scale_f32(result: AssetTable_Result, index: usize) -> f32 {
    match index {
        1 => result.get_scale_head(),
        2 => result.get_scale_neck(),
        3 => result.get_scale_torso(),
        4 => result.get_scale_shoulders(),
        5 => result.get_scale_arms(),
        6 => result.get_scale_hands(),
        7 => result.get_scale_legs(),
        8 => result.get_scale_feet(),
        9 => result.get_volume_bust(),
        10 => result.get_volume_abdomen(),
        11 => result.get_volume_torso(),
        12 => result.get_volume_base_arms(),
        13 => result.get_volume_base_legs(),
        14 => result.get_volume_scale_arms(),
        15 => result.get_volume_scale_legs(),
        16 => result.get_map_scale_all(),
        17 => result.get_map_scale_head(),
        18 => result.get_map_scale_wing(),
        _ => result.get_scale_all(),
    }
}

pub fn get_asset_table_color(entry: AssetTable, idx: usize) -> Color {
    match idx {
        1 => entry.grad_color(),
        2 => entry.skin_color(),
        3 => entry.toon_shadow_color(),
        4 => entry.mask_color100(),
        5 => entry.mask_color075(),
        6 => entry.mask_color050(),
        7 => entry.mask_color025(),
        _ => entry.hair_color(),
    }
}

pub fn get_asset_table_color_u8_slice(entry: AssetTable, idx: usize) -> [u8; 3]{
    let color = get_asset_table_color(entry, idx);
    [(color.r * 255.0) as u8, (color.g * 255.0) as u8, (color.b * 255.0) as u8]
}
pub fn get_asset_table_scale(entry: AssetTable, idx: usize) -> f32 {
    match idx {
        1 => entry.get_scale_head(),
        2 => entry.get_scale_neck(),
        3 => entry.get_scale_torso(),
        4 => entry.get_scale_shoulders(),
        5 => entry.get_scale_arms(),
        6 => entry.get_scale_hands(),
        7 => entry.get_scale_legs(),
        8 => entry.get_scale_feet(),
        9 => entry.get_volume_bust(),
        10 => entry.get_volume_abdomen(),
        11 => entry.get_volume_torso(),
        12 => entry.get_volume_arms(),
        13 => entry.get_volume_legs(),
        14 => entry.get_volume_scale_arms(),
        15 => entry.get_volume_scale_legs(),
        16 => entry.get_map_scale_all(),
        17 => entry.get_map_scale_head(),
        18 => entry.get_map_scale_wing(),
        _ => entry.get_scale_all(),
    }
}
pub fn get_asset_table_scale_u16(entry: AssetTable, idx: usize) -> u16 {
    let v = get_asset_table_scale(entry, idx) * 100.0;
    v as u16
}
pub fn get_result_scale_u16(result: AssetTable_Result, index: usize) -> u16 {
    let v = get_result_scale_f32(result, index) * 100.0;
    v as u16
}
pub fn try_find_accessory_model(result: AssetTable_Result, search_model: &str) -> Option<String> {
    result.m_accessories().items().iter()
        .filter(|a| !a.is_null())
        .filter_map(|a| crate::il2str(a.get_model()))
        .find(|model| model.contains(search_model))
}
pub fn try_find_acc_model_in_entry(entry: AssetTable, search_model: &str) -> Option<String> {
    entry.get_accessories().items().iter()
        .filter(|a| !a.is_null())
        .filter_map(|a| crate::il2str(a.get_model()))
        .find(|model| model.contains(search_model))
}
pub fn try_get_model_at_locator(result: AssetTable_Result, search_locator: &str) -> Option<String> {
    result.m_accessories().items().iter()
        .filter(|a| !a.is_null())
        .filter_map(|a| crate::il2str(a.get_model()).zip(crate::il2str(a.get_locator())))
        .find(|(_model, locator)| locator == search_locator)
        .map(|(model, _)| model.clone())
}
pub fn get_result_anim(result: AssetTable_Result, index: usize) -> Option<Il2CppString> {
    let s =
    match index {
        1 => result.m_talk_anim(),
        2 => result.m_demo_anim(),
        3 => result.m_hub_anim(),
        _ => result.m_info_anim()
    };
    if s.is_null() { None } else { Some(s) }
}
pub fn set_result_anim(result: AssetTable_Result, index: usize, anim: impl Into<Il2CppString>){
    match index {
        0 => result.set_m_info_anim(anim.into()),
        1 => result.set_m_talk_anim(anim.into()),
        2 => result.set_m_demo_anim(anim.into()),
        3 => result.set_m_hub_anim(anim.into()),
        _ => {}
    }
}