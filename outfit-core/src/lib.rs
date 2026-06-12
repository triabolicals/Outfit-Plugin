use std::sync::OnceLock;
pub use unity::prelude::*;
use engage::{
    spriteatlasmanager::FaceThumbnailStaticFields, gamedata::GamedataArray,
    keyhelp::KeyHelpData, proc::ProcInst,
};
use engage::combat::{CharacterAppearance, Kaneko};
use engage::ut::Ut;

#[allow(static_mut_refs, non_contiguous_range_endpoints)] mod data;
#[allow(static_mut_refs, non_contiguous_range_endpoints)]mod playerdata;
#[allow(static_mut_refs, non_contiguous_range_endpoints)] mod enums;
#[allow(static_mut_refs, non_contiguous_range_endpoints)]mod assets;
#[allow(static_mut_refs, non_contiguous_range_endpoints)] mod menu;
#[allow(static_mut_refs)] mod utils;
#[allow(static_mut_refs)] mod output;
#[allow(static_mut_refs)] mod shop;
#[allow(static_mut_refs)] mod unitasset;
mod photo;
mod localize;
mod capture;

pub use enums::*;
pub use data::*;
pub use playerdata::*;
pub use unitasset::*;
pub use output::*;
pub use utils::*;
pub use menu::*;
pub use shop::*;
pub use assets::*;
pub use assets::new_result_get_hash_code;
pub use data::dress::PersonalDressData;
pub use capture::reset_faces;
pub const VERSION: &'static str = "2.7.3c";
pub const GAME_USER_DATA_VERSION: i32 = 23;
pub const OUTPUT_ASSET_TABLE_DIR: &str = "sd:/engage/outfits/results/";
pub const OUTPUT_DATA: &str = "sd:/engage/outfits/data/";
pub const INPUT_DIR: &str = "sd:/engage/outfits/input/";
pub const CAPTURE_DIR: &str = "sd:/engage/outfits/capture/";
pub const THUMB_DIR: &str = "sd:/engage/outfits/capture/face/";
pub use menu::items::AssetType;

pub static OUTFIT_DATA: OnceLock<OutfitData> = OnceLock::new();

pub fn get_outfit_data() -> &'static OutfitData { OUTFIT_DATA.get_or_init(|| {OutfitData::init()}) }

#[unity::class("App", "StructTemplate`1")]
pub struct StructTemplate {}

#[unity::class("App", "StructDictionary`1")]
pub struct StructDictionary {
    pub key_list: &'static mut List<Il2CppString>,
    pub index_key: &'static mut Dictionary<'static, &'static Il2CppString, i32>,
    pub hash_key: &'static mut Dictionary<'static, i32, i32>,
}
#[repr(C)]
pub struct StructTemplateStaticFields{
    header: u64,
    pub dictionary: &'static mut StructDictionary,
}
fn photo_off(_proc: &ProcInst, _optional_method: OptionalMethod) {
    UnitAssetMenuData::get().mode = MenuMode::Inactive;
}
fn photo_on(_proc: &ProcInst, _optional_method: OptionalMethod) {
    UnitAssetMenuData::get().mode = MenuMode::PhotoGraph;
    UnitAssetMenuData::init_photo_profiles();
}

pub fn install_outfit_plugin(is_dvc: bool) -> bool {
    UnitAssetMenuData::get().is_dvc = is_dvc;
    if UnitAssetMenuData::get().init {
        UnitAssetMenuData::get().data.clear();
        return true;
    }
    skyline::install_hook!(appearance_create_from_result);
    let mut init = false;
    println!("Installing Outfit Plugin v{} ...", VERSION);
    OUTFIT_DATA.get_or_init(|| {
        init = true;
        let data = OutfitData::init();
        data
    });

    let _ = std::fs::create_dir_all(OUTPUT_ASSET_TABLE_DIR);
    let _ = std::fs::create_dir_all(OUTPUT_DATA);
    let _ = std::fs::create_dir_all(INPUT_DIR);
    let _ = std::fs::create_dir_all(CAPTURE_DIR);
    let _ = std::fs::create_dir_all(THUMB_DIR);

    let vtable = Il2CppClass::from_name("App", "GameUserData").unwrap().get_vtable_mut();
    vtable[4].method_ptr = game_user_data_version as _;
    vtable[12].method_ptr = game_user_data_on_deserialize as _;
    vtable[11].method_ptr = game_user_data_on_serialize as _;
    get_nested_virtual_methods_mut("App", "AssetTable", "Result", "GetHashCode")
        .map(|method|{ method.method_ptr = new_result_get_hash_code as _; });
    if let Some(class) = Il2CppClass::from_name("App", "PhotographTopSequence").ok() {
        if let Some(method) = class.get_virtual_method_mut("OnDispose") { method.method_ptr = photo_off as _; }
        if let Some(method) = class.get_virtual_method_mut("OnBind") { method.method_ptr = photo_on as _; }
    }
    if let Some(class) = Il2CppClass::from_name("App", "HubAccessoryRoom").ok() {
        if let Some(method) = class.get_virtual_method_mut("OnDispose") { method.method_ptr = room::CustomHubAccessoryRoom::on_dispose as _; }
    }
    if let Some(method) = Il2CppClass::from_name("App", "ShopUnitSelectMenuItemContent").ok()
        .and_then(|k| k.get_virtual_method_mut("Build"))
    {
        method.method_ptr = unitselect::shop_unit_select_menu_item_content_build as _;
    }
    if let Some(class) = Il2CppClass::from_name("App", "PhotographEditDisposMenu").ok() {
        if let Some(method) = class.get_virtual_method_mut("YCall") { method.method_ptr = photo::photograph_edit_dispos_menu_minus as _; }
    }
    if let Some(method) = Il2CppClass::from_name("App", "AccessoryMenuItemContent").ok()
        .and_then(|k| k.get_virtual_method_mut("BuildText"))
    {
        method.method_ptr = accessory_menu_item_content_build_text as _;
    }
    get_nested_virtual_methods_mut("App", "SortieUnitSelect", "UnitMenuItem", "YCall").map(|method| method.method_ptr = unit_item_y_call as _);
    get_nested_virtual_methods_mut("App", "MapUnitCommandMenu", "ItemMenuItem", "XCall").map(|method| method.method_ptr = unit_item_y_call as _);
    if let Some(klass) = Il2CppClass::from_name("App", "AccessoryShopChangeMenu").ok() {
        klass._2.actual_size = size_of::<CustomAssetMenu>() as u32;
        klass._2.instance_size = size_of::<CustomAssetMenu>() as u32;
    }

    skyline::patching::Patch::in_text(0x2173ba4).bytes(&[0x40, 0x01, 0x80, 0x52]).unwrap();
    skyline::patching::Patch::in_text(0x27b665c).bytes(&[0x01, 0x01, 0x80, 0x52]).unwrap();   // AccessoryEquipment Kind to 8
    skyline::patching::Patch::in_text(0x27b66d4).bytes(&[0x08, 0x01, 0x80, 0x52]).unwrap();
    skyline::patching::Patch::in_text(0x2166454).bytes(&[0x01, 0x20, 0x80, 0x52]).unwrap(); //Combat HierachyCache to 256
    sortie_menu_x_call_edit();
    UnitAssetMenuData::get().is_loaded = false;
    UnitAssetMenuData::get().init = true;
    UnitAssetMenuData::get().data.clear();
    if let Some(key) = KeyHelpData::try_get_mut("KHID_写真撮影_配置編集") {
        let y_button = KeyHelpData::instantiate().unwrap();
        y_button.button_index = 3;
        y_button.mid = "MID_MENU_ACCESSORY_SHOP_ACCESSORY".into();
        key.add(y_button);
    }
    let thumbs = &engage::spriteatlasmanager::FaceThumbnail::class().get_static_fields_mut::<FaceThumbnailStaticFields>().face_thumb;
    let s = thumbs.cache_table.entries.iter().filter(|i| i.key.is_some()).map(|c| c.key.unwrap().to_string()).collect::<Vec<String>>();

    s.iter().for_each(|i|{
        if let Some(sprite) = thumbs.cache_table.get_item(i.into()) {
            let o_key = format!("o_{}", i);
            let alt_key = format!("a_{}", i);
            thumbs.cache_table.add(o_key.as_str().into(), sprite);
            thumbs.cache_table.add(alt_key.as_str().into(), sprite);
        }
    });
    init
}
pub fn get_head_hair_colors(go: &GameObject) {
    if go.is_null() { return; }
    let data = UnitAssetMenuData::get();
    if data.is_preview {
        let update = data.preview.update;
        if update != 0{
            let colors: [&str; 6] = ["_BaseColor", "_BlackColor", "_DecalColor1", "_DecalColor2", "_DecalColor3", "_DecalColor4"];
            if update & 1 != 0 {
                data.preview.has_hair_acc = false;
                for x in 0..4 { data.preview.original_color[x] = 0; }
                if let Some(hair_go) = Kaneko::find_in_children(go.get_transform(), "c_spine1_jnt".into())
                    .or_else(|| Kaneko::find_in_children(go.get_transform(), "meshHairGP".into()))
                    .and_then(|m| m.get_game_object())
                {
                    if let Some(mt_hair) = get_material_from_object(hair_go, "MtHair ") {
                        let color = mt_hair.get_color(colors[0]);
                        data.preview.original_color[0] = (color.r * 255.0) as u8;
                        data.preview.original_color[1] = (color.g * 255.0) as u8;
                        data.preview.original_color[2] = (color.b * 255.0) as u8;
                        data.preview.original_color[3] = 1;
                    }
                    data.preview.has_hair_acc =
                        hair_go.get_components_in_children::<SkinnedMeshRenderer>(true).iter()
                            .any(|r| {
                                let name = r.get_name().to_string();
                                (name.contains("_Acc") && name.starts_with("h")) || name.starts_with("acc")
                            });
                }
                if let Some(m) = get_material_from_object(go, "MtHair2").or_else(|| get_material_from_object(go, "MtOdd")) {
                    let color = m.get_color(colors[0]);
                    data.preview.original_color[56] = (color.r * 255.0) as u8;
                    data.preview.original_color[56+1] = (color.g * 255.0) as u8;
                    data.preview.original_color[56+2] = (color.b * 255.0) as u8;
                    data.preview.original_color[56+3] = 1;
                }
                else { for x in 0..4 { data.preview.original_color[56+x] =0; } }
            }
            if update & 2 != 0 {
                for i in 0..24 { data.preview.original_color[32+i] =0; }
                if let Some(m) = get_material_from_object(go, "MtEye") {
                    for x in 0..6 {
                        let color = m.get_color(colors[x]);
                        data.preview.original_color[(8+x)*4] = (color.r * 255.0) as u8;
                        data.preview.original_color[(8+x)*4+1] = (color.g * 255.0) as u8;
                        data.preview.original_color[(8+x)*4+2] = (color.b * 255.0) as u8;
                    }
                }
            }
            data.preview.update = 0;
        }
    }
}

pub fn apply_preview_head_hair_color(this: &mut CharacterAppearance, go: &GameObject) {
    if go.is_null() { return; }
    let data = UnitAssetMenuData::get();
    let mut rgb: Option<[u8; 3]> = None;
    let colors: [&str; 6] = ["_BaseColor", "_BlackColor", "_DecalColor1", "_DecalColor2", "_DecalColor3", "_DecalColor4"];
    let data2 =
        if data.is_preview { Some(data.preview.preview_data.clone()) }
        else {
            UnitAssetMenuData::get_by_person_data(this.person_hash, false)
                .and_then(|p| p.profile.get(p.profile_index(false) as usize).cloned())
        };

    if let Some(data2) = data2 {
        for j in [2, 8, 9, 10, 11, 12, 13, 14] {
            rgb = None;
            let i = 4*j;
            if data.is_preview && data.preview.color_preview[i+3] == 1 {
                rgb = Some([data.preview.color_preview[i], data.preview.color_preview[i+1], data.preview.color_preview[i+2]]);
            }
            else {
                if data2.colors[j].values[3] != 0 {
                    rgb = Some([data2.colors[j].values[0], data2.colors[j].values[1], data2.colors[j].values[2]]);
                }
            }
            if let Some(rgb) = rgb.filter(|r| r[0] > 0 || r[1] > 0 || r[2] > 0) {
                let r = rgb[0] as f32 / 255.0;
                let g = rgb[1] as f32 / 255.0;
                let b = rgb[2] as f32 / 255.0;
                if j >= 8 &&  j < 14  {
                    if let Some(m) = get_mt_eye(go) { m.set_color(colors[j-8], Color::new(r, g, b, 1.0)); }
                }
                else if j == 2 {
                    go.get_components_in_children::<SkinnedMeshRenderer>(true).iter().for_each(|re|{
                        Ut::get_instance_materials2(re).iter().for_each(|m|{
                            if m.get_name().str_contains("MtSkin") {
                                m.set_float("_Makeup", 0.0);
                                m.set_color(colors[0], Color::new(r, g, b, 1.0));
                            }
                        });
                    });
                }
                else if j == 14 {
                    if let Some(m) = get_material_from_object(go, "MtHair2").or_else(|| get_material_from_object(go, "MtOdd")){
                        m.set_color(colors[0], Color::new(r, g, b, 1.0));
                    }
                }
            }
        }
        let flag = data2.flag;
        room::head_acc(go, flag & 64 != 0);
        room::hair_acc(go, flag & 16 != 0);
    }
}
fn get_mt_eye(go: &GameObject) -> Option<&'static &'static Material2> {
    go.get_component_in_children::<SkinnedMeshRenderer>(true).iter()
        .flat_map(|smr| Ut::get_instance_materials2(smr).iter())
        .find(|v| v.get_name().to_string().starts_with("MtEye"))
}

fn get_material_from_object(go: &GameObject, name: &str) -> Option<&'static &'static Material2> {
    go.get_components_in_children::<Renderer>(true).iter()
        .flat_map(|smr| Ut::get_instance_materials(smr).iter())
        .find(|v| v.get_name().to_string().contains(name))
        .or_else(||
            go.get_components_in_children::<SkinnedMeshRenderer>(true).iter()
                .flat_map(|smr| Ut::get_instance_materials2(smr).iter())
                .find(|v| v.get_name().to_string().contains(name))
        )
}
