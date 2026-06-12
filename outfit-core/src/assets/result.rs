use engage_il2cpp::app::{AssetTable_Result, IAssetTable_ResultMethods};
use engage_il2cpp::unity_engine::Color;

pub fn set_color_by_u8_slice(result: AssetTable_Result, idx: usize, v: [u8; 4]) {
    set_color_by_u8(result, idx, v[0], v[1], v[2]);
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