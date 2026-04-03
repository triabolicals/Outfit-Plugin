use std::sync::OnceLock;
use skyline::install_hook;
pub use unity::prelude::*;
use unity::system::{Dictionary, List};
pub use utils::Randomizer;
pub use data::dress::PersonalDressData;

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
use engage::gamedata::{GamedataArray};
use engage::keyhelp::KeyHelpData;
use engage::proc::ProcInst;

pub const VERSION: &'static str = "2.5.4";
pub const GAME_USER_DATA_VERSION: i32 = 23;
pub const OUTPUT_ASSET_TABLE_DIR: &str = "sd:/engage/outfits/results/";
pub const OUTPUT_DATA: &str = "sd:/engage/outfits/data/";
pub const INPUT_DIR: &str = "sd:/engage/outfits/input/";
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
/*
pub static mut WEAPON_KIND: i32 = 0;
fn plus(_this: &BasicMenuItem, optional_method: OptionalMethod) -> BasicMenuResult {
    let new = unsafe { WEAPON_KIND + 1};
    unsafe {
        WEAPON_KIND =     if new == 7 { 8 }
        else if new > 8 { 0 }
        else { new };
        println!("NEW KIND: {}", WEAPON_KIND);
    }
    BasicMenuResult::se_decide()
}
fn minus(_this: &BasicMenuItem, optional_method: OptionalMethod) -> BasicMenuResult {
    let new = unsafe { WEAPON_KIND - 1};
    unsafe {
        WEAPON_KIND = if new == 7 { 6 }
        else if new < 0 { 8 }
        else { new };
        println!("NEW KIND: {}", WEAPON_KIND);
    }
    BasicMenuResult::se_decide()
}
pub struct MapClassChangeDebug{}
impl MapClassChangeDebug {
    pub fn plus_call(this: &BasicMenuItem, optional_method: OptionalMethod) -> BasicMenuResult {
        Self::class_change(true);
        if let Some(unit) = MapMind::get_unit() { unit.reload_actor(); }
        BasicMenuResult::se_cursor()
    }
    pub fn minus_call(this: &BasicMenuItem, optional_method: OptionalMethod) -> BasicMenuResult {
        Self::class_change(false);
        if let Some(unit) = MapMind::get_unit() { unit.reload_actor(); }
        BasicMenuResult::se_decide()
    }

    pub fn class_change(plus: bool) -> BasicMenuResult{
        if let Some(unit) = MapMind::get_unit(){
            let job_count = JobData::get_count();
            let mut job_index = unit.job.parent.index;
            let mut count = 0;
            let change = if plus { 1 } else { -1 };
            let db = get_outfit_data();
            loop {
                job_index += change;
                if let Some(job_data) = JobData::try_index_get(job_index) {
                    if db.anims.job_anims.iter().any(|x| x.hash == job_data.parent.hash) || db.dress.transform.iter().any(|x| x.hash == job_data.parent.hash) {
                        unit.class_change(job_data);
                        println!("[Debug] {} Class Changed to: {}", unit.get_name(), Mess::get_name(job_data.jid));
                        add_items(unit, false);
                        UnitInfo::set_unit(UnitInfoSide::Left, None, false, false, false, None);
                        UnitInfo::set_unit(UnitInfoSide::Left, Some(unit), false, false, false, None);
                        // unit.auto_equip();
                        unit.reload_actor();
                        unit.set_hp(unit.get_capability(0, true));
                        return BasicMenuResult::se_cursor()
                    }
                }
                else { job_index = if plus { 0 } else { job_count } }
                count += 1;
                if count == job_count { return BasicMenuResult::se_miss(); }
            }
        }
        BasicMenuResult::new()
    }
}
pub fn add_items(unit: &mut Unit, auto_equip: bool) {
    let mut select_count = 0;
    let mut kinds = unit.job.weapons.iter().enumerate().filter(|(kind, x)| **x == 1).map(|(k, _)| k as i32).collect::<Vec<i32>>();
    if let Some(select_mask) = unit.job.get_selectable_weapon_mask(&mut select_count) {
        let rng = Random::get_game();
        let mut count = 0;
        let mut possible_kinds = vec![];
        for x in 1..10 {
            if select_mask.value & (1 << x) != 0 { possible_kinds.push(x); }
        }
        for _ in 0..9 {
            let len = possible_kinds.len();
            if len > 1 {
                let kind = possible_kinds.remove(rng.get_value(len as i32) as usize);
                unit.selected_weapon_mask.value |= 1 << kind;
                kinds.push(kind);
                count += 1;
            } else if len == 1 {
                unit.selected_weapon_mask.value |= 1 << possible_kinds[0];
                kinds.push(possible_kinds[0]);
                break;
            } else { break }
            if count == select_count { break; }
        }
    }
    unit.item_list.put_off_all_item();
    unit.selected_weapon_mask.value = 0;
    let item_list = ItemData::get_list().unwrap();
    kinds.iter().for_each(|kind| {
        let kind = *kind as u32;
        match kind {
            9 => {
                if let Some(item) = unit.job.mask_skills.find_sid("SID_竜石装備")
                    .or_else(|| unit.job.mask_skills.find_sid("SID_弾丸装備"))
                    .and_then(|skill|
                        item_list.iter()
                            .find(|x| x.kind == kind && x.equip_condition == Some(skill.sid) && x.flag.value & 128 == 0)) {
                    unit.item_list.add_item_no_duplicate(item);
                }
                else if let Some(item) = item_list.iter().
                    find(|x| x.kind == kind && x.equip_condition == None && x.flag.value & 128 == 0) { unit.item_list.add_item_no_duplicate(item); }
            }
            1..9 => {
                if let Some(item) = item_list.iter().find(|x| x.kind == kind && x.equip_condition == None && x.flag.value & 128 == 0) { unit.item_list.add_item_no_duplicate(item); }
            }
            _ => {}
        }
    });
    if auto_equip { unit.auto_equip(); }
    else {
        unit.item_list.unit_items.iter_mut().for_each(|x| {
            if let Some(x) = x.as_mut() { x.flags &= !1; }
        });
    }
}
pub fn get_all_god(menu_item: &BasicMenuItem, optional_method: OptionalMethod) -> BasicMenuResult {
    GodData::get_list().unwrap().iter().filter(|x| x.main_data.force_type == 0).for_each(|god|{
        engage::god::GodPool::create(god.main_data);
    });
    Force::get(ForceType::Player).unwrap().iter().for_each(|unit|{
        unit.private_skill.add_sid("SID_残像", SkillDataCategorys::Private, 0);
    });
    BasicMenuResult::se_cursor()
}
 */
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
    install_hook!(room::create_break_effect);
    println!("Creating Directories");
    let _ = std::fs::create_dir_all(OUTPUT_ASSET_TABLE_DIR);
    let _ = std::fs::create_dir_all(OUTPUT_DATA);
    let _ = std::fs::create_dir_all(INPUT_DIR);

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
    /*
    if let Some(k) = Il2CppClass::from_name("App", "MapUnitCommandMenu").ok()
        .and_then(|k| k.get_nested_types().iter().find(|c| c.get_name() == "ItemMenuItem".to_string()))
        .and_then(|s| Il2CppClass::from_il2cpptype(s.get_type()).ok())

    {
        k.get_virtual_method_mut("PlusCall").map(|m|m.method_ptr = MapClassChangeDebug::plus_call as _);
        k.get_virtual_method_mut("MinusCall").map(|m|m.method_ptr = MapClassChangeDebug::minus_call as _);
    }

     */
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
    println!("Installing Patches");
    skyline::patching::Patch::in_text(0x2173ba4).bytes(&[0x40, 0x01, 0x80, 0x52]).unwrap();
    skyline::patching::Patch::in_text(0x27b665c).bytes(&[0x01, 0x01, 0x80, 0x52]).unwrap();   // AccessoryEquipment Kind to 8
    skyline::patching::Patch::in_text(0x27b66d4).bytes(&[0x08, 0x01, 0x80, 0x52]).unwrap();
    skyline::patching::Patch::in_text(0x2166454).bytes(&[0x01, 0x20, 0x80, 0x52]).unwrap(); //Combat HierachyCache to 256
    sortie_menu_x_call_edit();
    UnitAssetMenuData::get().is_loaded = false;
    UnitAssetMenuData::get().init = true;
    UnitAssetMenuData::get().data.clear();
    println!("Outfit Plugin Installed!");
    if let Some(key) = KeyHelpData::try_get_mut("KHID_写真撮影_配置編集") {
        let y_button = KeyHelpData::instantiate().unwrap();
        y_button.button_index = 3;
        y_button.mid = "MID_MENU_ACCESSORY_SHOP_ACCESSORY".into();
        key.add(y_button);
    }
    init
}