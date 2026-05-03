use std::sync::OnceLock;
pub use unity::prelude::*;
use unity::system::{Dictionary, List};
use engage::{
    spriteatlasmanager::FaceThumbnailStaticFields, gamedata::GamedataArray,
    keyhelp::KeyHelpData, proc::ProcInst,
};

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
pub const VERSION: &'static str = "2.6.4b";
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
    let mut init = false;
    println!("Installing Outfit Plugin...");
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