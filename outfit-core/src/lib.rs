use std::sync::OnceLock;
pub use engage::prelude::*;
use engage::{
    app::{
        PhotographEditDisposMenu, PhotographTopSequence,
        IKeyHelpDataMethods, ISpriteAtlasManager_2, IStructDataArray_1Methods
    },
    system::collections::generic::{IDictionary_2Methods, IList_1Methods}
};
use skyline::install_hook;
use unity::Class;

#[allow(static_mut_refs, non_contiguous_range_endpoints)] mod data;
#[allow(static_mut_refs, non_contiguous_range_endpoints)]mod playerdata;
#[allow(static_mut_refs, non_contiguous_range_endpoints)] mod enums;
#[allow(static_mut_refs, non_contiguous_range_endpoints)]mod assets;
#[allow(static_mut_refs, non_contiguous_range_endpoints)] mod menu;
#[allow(static_mut_refs)] mod utils;
#[allow(static_mut_refs)] mod output;
#[allow(static_mut_refs)] mod shop;
#[allow(static_mut_refs)] mod unitasset;
mod localize;
mod capture;
mod photo;
mod model;

pub use enums::*;
pub use data::*;
pub use playerdata::*;
pub use unitasset::*;
pub use output::*;
pub use utils::*;
pub use menu::*;
pub use shop::*;
pub use assets::*;

pub use crate::{
    playerdata::deserialize_outfit_data,
    data::{PersonalDressDataFlags, dress::PersonalDressData},
    assets::{AssetConditions, AssetFlags},
    menu::items::AssetType,
    model::update_class_change_person,
    capture::reset_faces,
};

pub const VERSION: &'static str = "2.8.5c";
pub const GAME_USER_DATA_VERSION: i32 = 23;
pub const OUTPUT_ASSET_TABLE_DIR: &str = "sd:/engage/outfits/results/";
pub const OUTPUT_DATA: &str = "sd:/engage/outfits/data/";
pub const INPUT_DIR: &str = "sd:/engage/outfits/input/";
pub const CAPTURE_DIR: &str = "sd:/engage/outfits/capture/";
pub const THUMB_DIR: &str = "sd:/engage/outfits/capture/face/";

pub static OUTFIT_DATA: OnceLock<OutfitData> = OnceLock::new();

pub fn get_outfit_data() -> &'static OutfitData { OUTFIT_DATA.get_or_init(|| OutfitData::init()) }

fn photo_off(_proc: engage::app::ProcInst, _: OptionalMethod) {
    UnitAssetMenuData::get().is_preview = false;
    UnitAssetMenuData::get().mode = MenuMode::Inactive;
}
fn photo_on(_proc: engage::app::ProcInst, _optional_method: OptionalMethod) {
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
    println!("Installing Outfit Plugin v{} ...", VERSION);
    if UnitAssetMenuData::get().init {
        UnitAssetMenuData::get().data.clear();
        return true;
    }
    install_hook!(anim::unit_model_play_anim);
    // output_job_asset_data();
    if cobapi::injection::register::<CreateUnitInfoModel>().is_ok() {}
    if cobapi::injection::register::<CustomAssetMenu>().is_ok() {}
    if cobapi::injection::register::<CustomAssetMenuItem3>().is_ok() {}
    if cobapi::injection::register::<photo::CreatePhotographCharacter>().is_ok() {}
    override_vtable2("App", "AssetTable.Result", "GetHashCode", get_result_hash as _);
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
    vtable[14].method_ptr = photo_off as _;
    if let Some(y_call) =  PhotographEditDisposMenu::class().raw_mut().get_virtual_method_mut("YCall") {
        y_call.method_ptr = photo::photograph_edit_dispos_menu_minus as _;
    }
    override_vtable2("App", "ShopUnitSelectMenuItemContent", "Build", unitselect::shop_unit_select_menu_item_content_build as _);
    override_vtable2("App", "AccessoryMenuItemContent", "BuildText", accessory_menu_item_content_build_text as _);
    override_vtable2("App", "SortieUnitSelect.UnitMenuItem", "YCall", unit_item_y_call as _);
    override_vtable2("App", "MapUnitCommandMenu.ItemMenuItem", "XCall", unit_item_y_call as _);
    skyline::patching::Patch::in_text(0x2173ba4).bytes(&[0x40, 0x01, 0x80, 0x52]).unwrap();
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
    let s: Vec<String> = table.iter().filter_map(|(k, _v)| il2str(k)).collect();
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