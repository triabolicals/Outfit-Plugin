use std::sync::OnceLock;
use engage::{
    app::{IBasicMenu, IBasicMenuItem, IBasicMenuItemMethods, PhotographEditDisposMenu, PhotographTopSequence},
    List_1Ext,
    unity_engine::{IComponentMethods, IGameObjectMethods, IMaterialMethods, IObject_2Methods, SkinnedMeshRenderer},
    app::{IKeyHelpDataMethods, ISpriteAtlasManager_2, IStructDataArray_1Methods},
    system::collections::generic::{IDictionary_2Methods, IList_1Methods}
};
use unity::{injection::*, Cast, Class, ClassIdentity};
pub use engage::prelude::*;

#[allow(static_mut_refs, non_contiguous_range_endpoints)] mod data;
#[allow(static_mut_refs, non_contiguous_range_endpoints)]mod playerdata;
#[allow(static_mut_refs, non_contiguous_range_endpoints)] mod enums;
#[allow(static_mut_refs, non_contiguous_range_endpoints)]mod assets;
#[allow(static_mut_refs, non_contiguous_range_endpoints)] mod menu;
#[allow(static_mut_refs)] mod utils;
#[allow(static_mut_refs)] mod output;
#[allow(static_mut_refs)] mod shop;
#[allow(static_mut_refs)] mod unitasset;
// mod photo;
mod localize;
mod capture;
mod photo;

pub use enums::*;
pub use data::*;
pub use playerdata::*;
pub use unitasset::*;
pub use output::*;
pub use utils::*;
pub use menu::*;
pub use shop::*;
pub use assets::*;
pub use data::dress::PersonalDressData;
pub use capture::reset_faces;
pub use crate::assets::{AssetConditions, AssetFlags};

pub const VERSION: &'static str = "2.7.4";
pub const GAME_USER_DATA_VERSION: i32 = 23;
pub const OUTPUT_ASSET_TABLE_DIR: &str = "sd:/engage/outfits/results/";
pub const OUTPUT_DATA: &str = "sd:/engage/outfits/data/";
pub const INPUT_DIR: &str = "sd:/engage/outfits/input/";
pub const CAPTURE_DIR: &str = "sd:/engage/outfits/capture/";
pub const THUMB_DIR: &str = "sd:/engage/outfits/capture/face/";
pub use menu::items::AssetType;
pub use data::PersonalDressDataFlags;

use crate::photo::CreatePhotographCharacter;
use crate::room::CreateUnitInfoModel;

pub static OUTFIT_DATA: OnceLock<OutfitData> = OnceLock::new();

pub fn get_outfit_data() -> &'static OutfitData { OUTFIT_DATA.get_or_init(|| OutfitData::init()) }

fn photo_off(_proc: engage::app::ProcInst, _: unity::OptionalMethod) {
    UnitAssetMenuData::get().mode = MenuMode::Inactive;
}
fn photo_on(_proc: engage::app::ProcInst, _optional_method: unity::OptionalMethod) {
    UnitAssetMenuData::get().mode = MenuMode::PhotoGraph;
    UnitAssetMenuData::init_photo_profiles();
}

pub fn install_outfit_plugin(is_dvc: bool) -> bool {
    UnitAssetMenuData::get().is_dvc = is_dvc;
    let _ = std::fs::create_dir_all(OUTPUT_ASSET_TABLE_DIR);
    let _ = std::fs::create_dir_all(OUTPUT_DATA);
    let _ = std::fs::create_dir_all(INPUT_DIR);
    let _ = std::fs::create_dir_all(CAPTURE_DIR);
    let _ = std::fs::create_dir_all(THUMB_DIR);
    skyline::install_hooks!(appearance_create_from_result);
    println!("Installing Outfit Plugin v{} ...", VERSION);
    if UnitAssetMenuData::get().init {
        UnitAssetMenuData::get().data.clear();
        return true;
    }
    println!("Registering Classes...");
    if cobapi::injection::register::<CreateUnitInfoModel>().is_ok() {}
    if cobapi::injection::register::<CustomAssetMenu>().is_ok() {}
    if cobapi::injection::register::<CustomAssetMenuItem3>().is_ok() {}
    if cobapi::injection::register::<CreatePhotographCharacter>().is_ok() {}
    let klass = Class::lookup("App", "GameUserData");
    let vtable = klass.raw_mut().get_vtable_mut();
    vtable[4].method_ptr = game_user_data_version as _;
    vtable[12].method_ptr = game_user_data_on_deserialize as _;
    vtable[11].method_ptr = game_user_data_on_serialize as _;

    let mut init = false;
    OUTFIT_DATA.get_or_init(|| {
        init = true;
        let data = OutfitData::init();
        data
    });


    let vtable = PhotographTopSequence::class().raw_mut().get_vtable_mut();
    vtable[10].method_ptr = photo_off as _;
    vtable[11].method_ptr = photo_on as _;
    if let Some(y_call) =  PhotographEditDisposMenu::class().raw_mut().get_virtual_method_mut("YCall") {
        y_call.method_ptr = photo::photograph_edit_dispos_menu_minus as _;
    }
    get_virtual_methods_mut("App", "ShopUnitSelectMenuItemContent", "Build").map(|k| k.method_ptr = unitselect::shop_unit_select_menu_item_content_build as _);
    get_virtual_methods_mut("App", "AccessoryMenuItemContent", "BuildText").map(|k| k.method_ptr = accessory_menu_item_content_build_text as _);
    get_virtual_methods_mut("App", "SortieUnitSelect.UnitMenuItem", "YCall").map(|k| k.method_ptr = unit_item_y_call as _);
    get_virtual_methods_mut("App", "MapUnitCommandMenu.ItemMenuItem", "XCall").map(|k| k.method_ptr = unit_item_y_call as _);
    
    skyline::patching::Patch::in_text(0x2173ba4).bytes(&[0x40, 0x01, 0x80, 0x52]).unwrap();
    // skyline::patching::Patch::in_text(0x27b665c).bytes(&[0x01, 0x01, 0x80, 0x52]).unwrap();   // AccessoryEquipment Kind to 8
    // skyline::patching::Patch::in_text(0x27b66d4).bytes(&[0x08, 0x01, 0x80, 0x52]).unwrap();
    skyline::patching::Patch::in_text(0x2166454).bytes(&[0x01, 0x20, 0x80, 0x52]).unwrap(); //Combat HierachyCache to 256
    sortie_menu_x_call_edit();
    UnitAssetMenuData::get().is_loaded = false;
    UnitAssetMenuData::get().init = true;
    UnitAssetMenuData::get().data.clear();
    let help = engage::app::KeyHelpData::get("KHID_写真撮影_配置編集".into());
    if !help.is_null() {
        let new = engage::app::KeyHelpData::new();
        new.set_button_index(3i8);
        new.set_mid("MID_MENU_ACCESSORY_SHOP_ACCESSORY");
        help.add(new);
    }
    let table = engage::app::FaceThumbnail::s_face_thumb().m_cache_table();
    let s: Vec<String> = table.iter().filter_map(|(k, v)| il2str(k)).collect();
    s.iter().for_each(|k|{
        let (found, sprite) = table.try_get_value(k.as_str().into());
        if found && !sprite.is_null() {
            let o_key = format!("o_{}", k);
            let alt_key = format!("a_{}", k);
            table.add(o_key.into(), sprite);
            table.add(alt_key.into(), sprite);
        }
    });
    init
}
pub fn get_head_hair_colors(go: engage::unity_engine::GameObject) {
    if go.is_null() { return; }
    let data = UnitAssetMenuData::get();
    if data.is_preview {
        let update = data.preview.update;
        if update != 0{
            let colors: [&str; 6] = ["_BaseColor", "_BlackColor", "_DecalColor1", "_DecalColor2", "_DecalColor3", "_DecalColor4"];
            if update & 1 != 0 {
                data.preview.has_hair_acc = false;
                for x in 0..4 { data.preview.original_color[x] = 0; }
                for hair in ["c_spine1_jnt", "meshHairGP"]{
                    let h = engage::combat::Kaneko::find_in_children(go.get_transform(), hair);
                    if !h.is_null() {
                        let go = h.get_game_object();
                        data.preview.has_hair_acc =
                            get_skin_mesh_renderers(go).is_some_and(|arr|{
                                arr.iter().map(|r| unsafe { r.cast::<SkinnedMeshRenderer>() })
                                    . any(|r|{
                                        let name = r.get_name().to_rust_string();
                                        (name.contains("_Acc") && name.starts_with("h")) || name.starts_with("acc")
                                    })
                            });
                        if let Some(mt_hair) = get_material_from_go(go, "MtHair") {
                            let color = mt_hair.get_color_2(colors[0]);
                            data.preview.original_color[0] = (color.r * 255.0) as u8;
                            data.preview.original_color[1] = (color.g * 255.0) as u8;
                            data.preview.original_color[2] = (color.b * 255.0) as u8;
                            data.preview.original_color[3] = 1;
                            break;
                        }
                    }
                }
                if let Some(m) = get_material_from_go(go, "MtHair2").or_else(|| get_material_from_go(go, "MtOdd")) {
                    let color = m.get_color_2(colors[0]);
                    data.preview.original_color[56] = (color.r * 255.0) as u8;
                    data.preview.original_color[56+1] = (color.g * 255.0) as u8;
                    data.preview.original_color[56+2] = (color.b * 255.0) as u8;
                    data.preview.original_color[56+3] = 1;
                }
                else { for x in 0..4 { data.preview.original_color[56+x] =0; } }
            }
            if update & 2 != 0 {
                for i in 0..24 { data.preview.original_color[32+i] =0; }
                if let Some(m) = get_material_from_go(go, "MtEye") {
                    for x in 0..6 {
                        let color = m.get_color_2(colors[x]);
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

pub fn apply_preview_head_hair_color(this: engage::combat::CharacterAppearance, go: engage::unity_engine::GameObject) {
    if go.is_null() { return; }
    let data = UnitAssetMenuData::get();
    let mut rgb: Option<[u8; 3]> = None;
    let colors: [&str; 6] = ["_BaseColor", "_BlackColor", "_DecalColor1", "_DecalColor2", "_DecalColor3", "_DecalColor4"];
    let data2 =
        if data.is_preview { Some(data.preview.preview_data.clone()) }
        else {
            let hash = unity::field_get_value_at_offset::<i32>(this, 0xd4);
            UnitAssetMenuData::get_by_person_data(hash, false).and_then(|p| p.profile.get(p.profile_index(false) as usize).cloned())
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
                    if let Some(m) = get_mt_eye(go) {
                        m.set_color_2(colors[j-8], engage::unity_engine::Color{r, g, b, a: 1.0});
                    }
                }
                else if j == 2 {
                    if let Some(arr) = get_skin_mesh_renderers(go) {
                        arr.iter()
                            .map(|r| unsafe { r.cast::<SkinnedMeshRenderer>() })
                            .for_each(|smr| {
                                engage::app::Ut::get_instance_materials(smr).iter().for_each(|m| {
                                    if m.get_name().to_rust_string().contains("MtSkin") {
                                        m.set_float("_Makeup", 0.0);
                                        m.set_color_2(colors[0], engage::unity_engine::Color { r, g, b, a: 1.0 });
                                    }
                                });
                            });
                    }
                }
                else if j == 14 {
                    if let Some(m) = get_material_from_go(go, "MtHair2").or_else(|| get_material_from_go(go, "MtOdd")){
                        m.set_color_2(colors[0], engage::unity_engine::Color{r, g, b, a: 1.0});
                    }
                }
            }
        }
        let flag = data2.flag;
        room::head_acc(go, flag & 64 != 0);
        room::hair_acc(go, flag & 16 != 0);
    }
}
fn get_mt_eye(go: engage::unity_engine::GameObject) -> Option<engage::unity_engine::Material>{
    get_material_from_go(go, "MtEye")
}

fn get_material_from_go(go: engage::unity_engine::GameObject, name: &str) -> Option<engage::unity_engine::Material> {
    if let Some(arr) = get_skin_mesh_renderers(go) {
        arr.iter()
            .map(|r| unsafe { r.cast::<SkinnedMeshRenderer>() })
            .flat_map(|r| engage::app::Ut::get_instance_materials(r).iter())
            .find(|m| m.get_name().to_rust_string().contains(name))
    }
    else { None }
}