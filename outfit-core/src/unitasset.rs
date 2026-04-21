use std::cmp::PartialEq;
use std::fs::{read_dir, read_to_string};
use engage::{
    unit::*,
    gamedata::{Gamedata, GodData, PersonData, assettable::AssetTableResult},
    gameuserdata::GameUserData,
    menu::BasicMenuResult
};
use engage::gamedata::assettable::AssetTableStaticFields;
use engage::sortie::SortieSelectionUnitManager;
use engage::util::try_get_instance;
pub use crate::playerdata::*;
use crate::assets::unit_dress_gender;
use crate::{get_outfit_data, AssetConditions, AssetType, Mount, PhotoCameraControl};
use crate::data::room::hub_room_set_by_result;

mod load;
pub use load::*;
use crate::anim::AnimData;
use crate::data::unitselect::{UnitSelect, UnitSelectList};
use crate::room::ReloadType;

pub static mut UNIT_ASSET: UnitAssetMenuData = UnitAssetMenuData::default();

#[derive(PartialEq, Copy, Clone)]
pub enum PreviewAsset {
    PreviewAsset(AssetType),
    Scale(u8),
    NoAsset,
}
#[derive(PartialEq, Copy, Clone, Default)]
pub enum MenuMode {
    #[default] Inactive,
    Shop,
    UnitInfo,
    PhotoGraph,
}
#[derive(Clone)]
pub struct UnitAssetPreview {
    pub person: i32,
    pub gender: i32,
    pub preview_data: PlayerOutfitData,
    pub selected_profile: i32,
    pub original_scaling: [u16; 20],
    pub color_preview: [u8; 32],
    pub scale_preview: [u16; 20],
    pub original_color: [u8; 32],
    pub original_assets: [i32; 20],

}
impl UnitAssetPreview {
    pub const fn new() -> Self {
        Self {
            person: 0, gender: 0,
            preview_data:
            PlayerOutfitData::new(),
            selected_profile: 0,
            original_color: [0; 32],
            scale_preview: [0; 20],
            original_scaling: [0; 20],
            color_preview: [0; 32],
            original_assets: [0; 20],
        }
    }
}


pub struct UnitAssetMenuData {
    pub data: Vec<UnitAssetData>,
    pub preview: UnitAssetPreview,
    pub loaded_data: UnitAssetLoader,
    pub is_preview: bool,
    pub mode: MenuMode,
    pub output: bool,
    pub is_hub: bool,
    pub is_shop_combat: bool,
    pub preview_accessory: bool,
    pub name_set: bool,
    pub god_mode: bool,
    pub is_dvc: bool,
    pub is_loaded: bool,
    pub is_changed: bool,
    pub init: bool,
    pub debug: bool,
    pub facial: usize,
    pub menu_adj: f32,
    pub control: PhotoCameraControl,
    pub photo_profiles: Vec<PlayerOutfitData>,
    pub unit_select: UnitSelectList,
}

pub enum LoadResult {
    Success,
    NoFiles,
    MissingDirectory,
}
#[derive(PartialEq, Copy, Clone)]
pub enum ReloadPreview {
    Scale(i32),
    Color(i32),
    Preset(usize),
    NoScaleColor,
    LoadedData,
    Forced,
}
impl UnitAssetMenuData {
    pub fn is_photo_graph() -> bool { Self::get().mode == MenuMode::PhotoGraph }
    pub fn is_unit_info() -> bool { Self::get().mode == MenuMode::UnitInfo }
    pub fn is_shop() -> bool { Self::get().mode == MenuMode::Shop }
    pub fn add_data(&mut self, data: UnitAssetData) {
        if self.data.iter().find(|v| v.person == data.person).is_none() {
            self.data.push(data);
        }
    }
    pub fn init_photo_profiles() {
        let data = Self::get();
        data.photo_profiles.clear();
        data.data.iter().for_each(|x|{
            let hash = x.person;
            if let Some(profile) = x.profile.get(x.profile_index(false) as usize) {
                let mut p = profile.clone();
                p.flag &= !128;
                p.break_body = hash;
                data.photo_profiles.push(p);
            }
        });
        data.mode = MenuMode::PhotoGraph;
    }
    pub fn get_result() -> &'static mut AssetTableResult {
        let data = Self::get();
        let hub = data.mode != MenuMode::UnitInfo;
        match data.mode {
            MenuMode::Shop => { data.unit_select.get_result(hub) }
            _ => {
                let select = UnitSelect {
                    hash: data.preview.person,
                    god: data.god_mode,
                    recruited: false,
                    female: data.preview.gender == 2
                };
                let result = select.get_result(hub);
                if data.mode == MenuMode::UnitInfo {
                    AnimData::remove(result, true, true);
                    result.body_anim = result.hub_anims;
                    result.info_anims = result.hub_anims;
                    result.talk_anims = None;
                    result.demo_anims = None;
                    result.left_hand = "null".into();
                    result.right_hand = "null".into();
                    result.replace(2);
                }
                result
            }
        }
    }
    pub fn get_current_dress_gender() -> i32 { Self::get_preview().gender }
    pub fn get_gender(alt: bool) -> i32 {
        let gender = Self::get_current_dress_gender();
        if !alt { gender } else if gender == 2 { 1 } else { 2 }
    }
    pub fn get() -> &'static mut UnitAssetMenuData { unsafe { &mut UNIT_ASSET } }
    pub fn get_unit() -> Option<&'static mut Unit> {
        if Self::is_shop(){ PersonData::try_get_hash(Self::get().preview.person).and_then(|p| UnitPool::get_from_person(p, false)) }
        else {
            if GameUserData::get_sequence() != 3 {
                if try_get_instance::<SortieSelectionUnitManager>().is_some_and(|v| v.unit.is_some()) { Some(SortieSelectionUnitManager::get_unit()) }
                else { None }
            }
            else { engage::map::mind::MapMind::get_unit() }
        }

    }
    const fn default() -> Self {
        Self {
            mode: MenuMode::Inactive,
            data: Vec::new(),
            photo_profiles: Vec::new(),
            is_dvc: false,
            preview: UnitAssetPreview::new(),
            is_shop_combat: false,
            preview_accessory: false,
            output: false,
            name_set: false,
            is_preview: false,
            is_hub: false,
            god_mode: false,
            is_loaded: false,
            is_changed: false,
            loaded_data: UnitAssetLoader::new(),
            init: false,
            debug: false,
            facial: 0,
            menu_adj: 0.0,
            control: PhotoCameraControl::default(),
            unit_select: UnitSelectList::new(),
        }
    }
    pub fn set_preview(data: &PlayerOutfitData){
        let preview = Self::get_preview();
        preview.preview_data = data.clone();
        for x in 0..16 { preview.scale_preview[x] = data.scale[x] }
        for x in 0..8 {
            for y in 0..4 { preview.color_preview[4*x+y] = data.colors[x].values[y]; }
        }
    }
    pub fn get_preview() -> &'static mut UnitAssetPreview { &mut Self::get().preview }
    pub fn set_profile(index: i32) {
        let data = Self::get();
        let (hash, current_profile) = (data.preview.person, data.preview.selected_profile);
        if current_profile == index { return; }
        if let Some(d) = data.data.iter_mut().find(|x| x.person == hash) {
            if let Some(p) = d.profile.get_mut(current_profile as usize){
                *p = data.preview.preview_data.clone();
            }
            if let Some(new_profile) = d.profile.get(index as usize) {
                for x in 0..8 {
                    for r in 0..4 {
                        data.preview.color_preview[4*x+r] = new_profile.colors[x].values[r];
                    }
                }
                for x in 0..16 {
                    let v = new_profile.scale[x];
                    data.preview.scale_preview[x] = if v == 0 { data.preview.original_scaling[x] } else { v };
                }
                data.preview.selected_profile = index;
                data.preview.preview_data = new_profile.clone();
            }
        }
    }
    pub fn get_current_asset_data() -> Option<&'static mut UnitAssetData> {
        let person = Self::get_preview().person;
        let data = Self::get();
        data.data.iter_mut().find(|x| x.person == person)
    }
    pub fn get_by_person_data(hash: i32, create: bool) -> Option<&'static UnitAssetData> {
        let menu = Self::get();
        if create {
            if menu.data.iter().find(|x| x.person == hash).is_none() { menu.data.push(UnitAssetData::new_hash(hash, false)); }
        }
        menu.data.iter().find(|x| x.person == hash)
    }
    pub fn get_unit_data(unit: &Unit) -> Option<&UnitAssetData>  {
        Self::get_by_person_data(unit.person.parent.hash, false).or_else(||{
            if (unit.force.is_some_and(|x| (1 << x.force_type) & 25 != 0) && unit.status.value & 35184372088832 == 0) || unit.person.is_hero() {
                Self::get_by_person_data(unit.person.parent.hash, true)
            }
            else { None }
        })
    }
    pub fn set_god(god: &GodData){
        let hash = god.parent.hash;
        Self::set_by_hash(hash);
    }
    pub fn set_by_hash(person: i32) -> bool {
        let menu = Self::get();
        let mut engaged = false;
        let gender;
        let photo = menu.mode == MenuMode::PhotoGraph;
        if let Some(person) = PersonData::try_get_hash(person) {
            menu.god_mode = false;
            if let Some(unit) = UnitPool::get_from_person(person, false) {
                engaged = unit.status.value & 8388608 != 0;
                gender = unit_dress_gender(unit);
            }
            else {
                gender =
                    if person.flag.value & 32 != 0 { if person.gender == 2 { 1 } else { 2 } }
                    else { if person.gender == 2 { 2 } else { 1 } };
            }
        }
        else if let Some(god) = GodData::try_get_hash(person) {
            menu.god_mode = true;
            gender =
                if god.is_hero() { UnitPool::get_hero(false).map(|u| unit_dress_gender(u)).unwrap_or(god.female + 1) }
                else { god.female + 1 };
        }
        else { return false; }
        let s = GameUserData::get_sequence();
        if photo {
            if let Some(data) = menu.photo_profiles.iter().find(|x| x.break_body == person).cloned(){ menu.preview.preview_data = data; }
            else if let Some(data) = menu.data.iter().find(|x| x.person == person){
                let index = data.profile_index(false) as usize;
                let mut d = data.profile[index].clone();
                d.break_body = person;
                menu.photo_profiles.push(d.clone());
                menu.preview.preview_data = d;
            }
        }
        else {
            if let Some(hash) = PersonData::try_get_hash(person).map(|v| v.parent.hash)
                .or_else(|| GodData::try_get_hash(person).map(|v| v.parent.hash))
            {
                if let Some(data) = Self::get_by_person_data(hash, true) {
                    let index = if s != 4 { if engaged && !menu.god_mode { 1 } else { 0 } } else { 2 };
                    menu.preview.selected_profile = index;
                    let profile = data.set_profile[index as usize];
                    menu.preview.preview_data = data.profile[profile as usize].clone();
                }
                else { menu.preview.selected_profile = -1; }
            }
            else { return false; }
        }
        menu.preview.person = person;
        menu.preview.gender = gender;
        menu.name_set = true;

        let result = Self::get_result();
        Self::set_original_assets();
        for x in 0..8 {
            menu.preview.color_preview[x * 4] = if result.unity_colors[x].r >= 1.0 { 255 } else { (result.unity_colors[x].r * 255.5) as u8 };
            menu.preview.color_preview[x * 4 + 1] = if result.unity_colors[x].g >= 1.0 { 255 } else { (result.unity_colors[x].g * 255.5) as u8 };
            menu.preview.color_preview[x * 4 + 2] = if result.unity_colors[x].b >= 1.0 { 255 } else { (result.unity_colors[x].b * 255.5) as u8 };
            menu.preview.color_preview[x * 4 + 3] = if result.unity_colors[x].a >= 1.0 { 255 } else { (result.unity_colors[x].a * 255.5) as u8 };
        }
        for x in 0..16 { menu.preview.scale_preview[x] = (result.scale_stuff[x] * 100.0) as u16; }
        if !photo { hub_room_set_by_result(Some(result), ReloadType::ForcedUpdate); }

        true
    }
    pub fn set_unit(unit: &Unit) -> bool { Self::set_by_hash(unit.person.parent.hash) }
    pub fn get_shop_unit() -> Option<&'static mut Unit> {
        let data = Self::get();
        if data.god_mode { None }
        else { PersonData::try_get_hash(data.preview.person).and_then(|p| UnitPool::get_from_person_force_mask(p, -1)) }
    }
    pub fn reload_unit(kind: ReloadPreview, forced: bool, result: Option<&mut AssetTableResult>) -> BasicMenuResult {
        let data = Self::get();
        if data.is_changed || forced  {
            match kind {
                ReloadPreview::Color(kind) => {
                    let result = result.unwrap_or(Self::get_result());
                    let mut color: i32 = 0;
                    for x in 0..3 { color += data.preview.color_preview[4*kind as usize + x] as i32; }
                    if color > 0 {
                        result.unity_colors[kind as usize].r = data.preview.color_preview[4*kind as usize] as f32 / 255.0;
                        result.unity_colors[kind as usize].g = data.preview.color_preview[4*kind as usize+1] as f32 / 255.0;
                        result.unity_colors[kind as usize].b = data.preview.color_preview[4*kind as usize+2] as f32 / 255.0;
                    }
                    hub_room_set_by_result(Some(result), ReloadType::ColorScale);
                }
                ReloadPreview::Scale(_kind) => { hub_room_set_by_result(result, ReloadType::Scale); }
                ReloadPreview::Preset(index) => {
                    let result = result.unwrap_or(Self::get_result());
                    let db = get_outfit_data();
                    if let Some(appearance) = db.dress.personal.get(index) {
                        appearance.apply_appearance(result, 2, false, None, &db.hashes, true);
                        result.ride_dress_model = None;
                        result.ride_model = None;
                        result.left_hand = "null".into();
                        result.right_hand = "null".into();
                        result.body_anim = Some(
                            if db.get_dress_gender(result.dress_model) == Gender::Male { "AOC_Hub_Hum0M" }
                            else { "AOC_Hub_Hum0F" }.into()
                        );
                        hub_room_set_by_result(Some(result), ReloadType::ForcedUpdate);
                    }
                }
                ReloadPreview::LoadedData => {
                    let result = result.unwrap_or(Self::get_result());
                    let menu = UnitAssetMenuData::get();
                    if let Some(loaded) = menu.loaded_data.selected_index.and_then(|i| menu.loaded_data.loaded_data.get_mut(i as usize)) {
                        let flag = loaded.data.flag;
                        loaded.data.flag |= 193;
                        loaded.data.set_result(result, 2, false, false);
                        loaded.data.flag = flag;
                    }
                    hub_room_set_by_result(Some(result), ReloadType::ForcedUpdate);
                }
                _ => {}
            }
            data.is_changed = false;
            BasicMenuResult::se_cursor()
        }
        else { BasicMenuResult::new() }
    }
    pub fn get_flag() -> i32 { Self::get().preview.preview_data.flag }
    pub fn set_flag(flag: i32) {
        let menu = Self::get();
        menu.preview.preview_data.flag = flag;
    }
    pub fn toggle_unit_flag(flag: i32) {
        let hash = Self::get_preview().person;
        if let Some(data) = Self::get().data.iter_mut().find(|x| x.person == hash) {
            data.flag ^= flag;
        }
    }
    pub fn toggle_profile_flag(flag: i32) { Self::get_preview().preview_data.flag ^= flag; }
    pub fn commit() {
        let menu = Self::get();
        let preview = Self::get_preview();
        if Self::is_photo_graph() {
            if let Some(p) = menu.photo_profiles.iter_mut().find(|x| x.break_body == preview.person) {
                *p = preview.preview_data.clone();
                p.break_body = preview.person;
            }
        }
        else {
            let hash = preview.person;
            let new_data = preview.preview_data.clone();
            let index = preview.selected_profile as usize;
            if let Some(data) = menu.data.iter_mut().find(|x| x.person == hash ){
                if let Some(d) = data.profile.get_mut(index) {
                    *d = new_data.clone();
                }
            }
        }

    }
    pub fn get_person_flag() -> i32 {
        let hash = Self::get_preview().person;
        Self::get().data.iter().find(|x| x.person == hash ).map(|x| x.flag).unwrap_or(0)
    }
    pub fn set_assets(result: &mut AssetTableResult, unit: &Unit, asset_conditions: &AssetConditions) {
        if !result.body_model.is_null() { if result.body_model.str_contains("AT") { return; } }
        if !result.dress_model.is_null() { if result.dress_model.str_contains("AT") { return; } }
        let mode = asset_conditions.mode;
        let menu = Self::get();
        let is_preview =  menu.is_preview;
        let is_engaged = unit.status.value & 8388608 != 0;
        let is_photo = menu.mode == MenuMode::PhotoGraph;
        if is_preview {
            if is_photo  { menu.preview.preview_data.set_result(result, mode, is_engaged, false); }
            else {
                if let Some(loaded) = menu.loaded_data.selected_index.and_then(|i| menu.loaded_data.loaded_data.get_mut(i as usize)) {
                    let flag = loaded.data.flag;
                    loaded.data.flag |= 193;
                    loaded.data.set_result(result, mode, is_engaged, false);
                    loaded.data.flag = flag;
                }
                else {
                    menu.preview.preview_data.set_result(result, mode, is_engaged, false);
                }
            }
        }
        else if is_photo {
            if let Some(data) = menu.photo_profiles.iter().find(|x| x.break_body == unit.person.parent.hash) {
                data.set_result(result, 2, false, false);
            }
        }
        else if let Some(data) = menu.data.iter().find(|s| s.person == unit.person.parent.hash){
            data.set_result(result, mode, is_engaged, asset_conditions.broken);
        }
    }
    pub fn set_god_assets(result: &mut AssetTableResult, mode: i32, god: &GodData, darkness: bool) {
        let menu = Self::get();
        if UnitAssetMenuData::is_photo_graph()  {
            if menu.is_preview {
                menu.preview.preview_data.set_result(result, 2, darkness, false);
            }
            else if let Some(data) = menu.photo_profiles.iter().find(|x| x.break_body == god.parent.hash) {
                data.set_result(result, 2, darkness, false);
            }
            return;
        }
        if let Some(data) = menu.data.iter().find(|s| s.person == god.parent.hash){
            data.set_result(result, mode, darkness, false)
        }
    }
    pub fn get_current_unit_hash(kind: AssetType) -> i32 {
        let preview = Self::get_preview();
        match kind {
            AssetType::Body => {
                let body = preview.preview_data.ubody;
                if body == 0 { preview.original_assets[0] } else { body }
            }
            AssetType::Head => {
                let head = preview.preview_data.uhead;
                if head == 0 { preview.original_assets[1] } else { head }
            }
            AssetType::Hair => {
                let hair = preview.preview_data.uhair;
                if hair == 0 { preview.original_assets[2] } else { hair }
            }
            AssetType::Acc(slot) => {
                let acc = preview.preview_data.acc[slot as usize];
                if acc == 0 { preview.original_assets[5+slot as usize] } else { acc }
            }
            AssetType::AOC(slot) => {
                let aoc = preview.preview_data.aoc[slot as usize];
                if aoc == 0 { preview.original_assets[10 + slot as usize] } else { aoc }
            }
            AssetType::Mount(slot)=> { preview.preview_data.mount[slot as usize] }
            AssetType::Voice => {
                let voice = preview.preview_data.voice;
                if voice == 0 { preview.original_assets[14] } else { voice }
            }
            AssetType::Rig => {
                let rig = preview.preview_data.rig;
                if rig == 0 { preview.original_assets[15] } else { rig }
            }
            AssetType::ColorPreset(kind) => {
                let mut color = 0;
                let mut original = 0;
                for x in 0..3 {
                    color += (preview.preview_data.colors[kind as usize].values[x] << 8*x) as i32;
                    original += (preview.original_color[4*kind as usize + x] << 8*x) as i32;
                }
                if color == 0 { original } else { color }
            }
        }

    }
    pub fn get_current_scale(index: i32) -> u16 {
        if index >= 16 { return 0; }
        let menu = Self::get_preview();
        let v = menu.preview_data.scale[index as usize];
        if v == 0 { menu.original_scaling[index as usize] }
        else { v }
    }
    pub fn get_current_color(color_index: i32, rgb: i32) -> u8 {
        if color_index >= 8 || rgb > 3 { return 0; }
        let menu = Self::get_preview();
        menu.preview_data.colors[color_index as usize].values[rgb as usize]
    }
    pub fn set_current_scale(index: i32, value: u16) {
        if index >= 16 { return; }
        let menu = Self::get_preview();
        let value = if value > 1000 { menu.original_scaling[index as usize] } else { value };
        menu.scale_preview[index as usize] = value;
        menu.preview_data.scale[index as usize] = value;
    }
    pub fn set_current_color(color_index: i32, rgb: i32, value: u8){
        if color_index >= 8 || rgb >= 4 { return; }
        let menu = Self::get_preview();
        menu.preview_data.colors[color_index as usize].values[rgb as usize] = value;
    }
    pub fn get_original_color_str(color_index: i32) -> String {
        let i =  4*color_index as usize;
        let menu = &Self::get_preview().original_color;
        format!("{}/{}/{}", menu[i], menu[i+1], menu[i+2])
    }
    pub fn get_preview_color_str(color_index: i32) -> String {
        let i =  4*color_index as usize;
        let menu = &Self::get_preview().color_preview;
        format!("{}/{}/{}", menu[i], menu[i+1], menu[i+2])
    }
    pub fn get_set_color_str(color_index: i32) -> String {
        let menu = &Self::get_preview().preview_data.colors[color_index as usize];
        format!("{}/{}/{}", menu.values[0], menu.values[1], menu.values[2])
    }
    pub fn set_original_assets() -> (Vec<i32>, Vec<i32>){
        let sf = AssetTableStaticFields::get();
        let flags = &sf.condition_flags;
        let db = get_outfit_data();
        let menu = Self::get_preview();
        let mut modes: (Vec<i32>, Vec<i32>) = (vec![], vec![]);
        menu.original_assets.iter_mut().for_each(|a| {*a = 0});
        menu.original_scaling.iter_mut().for_each(|s| {*s = 0});
        menu.original_color.iter_mut().for_each(|c| {*c = 0});
        menu.original_assets[16] = -1;
        let photo = UnitAssetMenuData::is_photo_graph();
        for mode in 1..3 {
            sf.search_lists[mode].iter().filter(|a| a.condition_indexes.test(flags)).for_each(|entry|{
                if mode == 1 {
                    modes.0.push(entry.parent.index);
                    if let Some(obody) = entry.body_model.and_then(|h| db.try_get_asset_hash(h)) { menu.original_assets[3] = obody; }
                    if let Some(ohair) = entry.hair_model.and_then(|h| db.try_get_asset_hash(h)) { menu.original_assets[4] = ohair; }
                }
                else if mode == 2 {
                    modes.1.push(entry.parent.index);
                    if let Some(body) = entry.dress_model.and_then(|h| db.try_get_asset_hash(h)) {
                        menu.original_assets[0] = body;
                    }
                    if let Some(head) = entry.head_model.and_then(|h| db.try_get_asset_hash(h)) { menu.original_assets[1] = head; }
                    if let Some(hair) = entry.accessory_list.list.iter()
                        .find(|x| x.model.is_some_and(|h| h.str_contains("Hair") && !h.str_contains("null")))
                        .and_then(|a| a.model.and_then(|a| db.try_get_asset_hash(a)))
                        .or_else(||entry.hair_model.and_then(|h| db.try_get_asset_hash(h)))
                    {
                        menu.original_assets[2] = hair;
                    }
                    for xx in 0..5 {
                        if let Some(acc) = entry.accessory_list.list.iter().find(|x| x.locator.is_some_and(|x| x.str_contains(ACC_LOC[xx])))
                            .and_then(|a| a.model.and_then(|a| db.try_get_asset_hash(a)))
                        {
                            menu.original_assets[5+xx] = acc;

                        }
                    }
                    if let Some(rig) = entry.body_model.and_then(|h| db.try_get_asset_hash(h)) { menu.original_assets[15] = rig; }
                }
                if let Some(ride) = entry.ride_dress_model.as_ref() {
                    let mount = Mount::from(ride.to_string().as_str()) as i32 - 1;
                    if menu.original_assets[16] < 0 && mount >= 0 { menu.original_assets[16] = mount; }
                }
                for x in 0..8 {
                    if (entry.unity_colors[x].r + entry.unity_colors[x].g + entry.unity_colors[x].b) > 0.0 {
                        menu.original_color[4*x] = (entry.unity_colors[x].r * 255.0) as u8;
                        menu.original_color[4*x+1] = (entry.unity_colors[x].g * 255.0) as u8;
                        menu.original_color[4*x+2] = (entry.unity_colors[x].b * 255.0) as u8;
                        menu.original_color[4*x+3] = (entry.unity_colors[x].a * 255.0) as u8;
                    }
                }
                for x in 0..9 {
                    if entry.scale_stuff[x] > 0.0 { menu.original_scaling[x] = (entry.scale_stuff[x] * 100.0) as u16; }
                }
                menu.original_scaling[9] = (entry.scale_stuff[11] * 100.0) as u16;
                menu.original_scaling[10] = (entry.scale_stuff[12] * 100.0) as u16;
                menu.original_scaling[11] = (entry.scale_stuff[13] * 100.0) as u16;
                menu.original_scaling[12] = (entry.scale_stuff[9] * 100.0) as u16;
                menu.original_scaling[13] = (entry.scale_stuff[10] * 100.0) as u16;
                for x in 14..19 {
                    if entry.scale_stuff[x] > 0.0 {
                        menu.original_scaling[x] = (entry.scale_stuff[x] * 100.0) as u16;
                    }
                }
                if let Some(voice) = entry.voice.and_then(|h| db.try_get_asset_hash(h)) { menu.original_assets[14] = voice; }
                if let Some(aoc) = entry.info_anim.and_then(|a| db.try_get_asset_hash(a)) { menu.original_assets[10] = aoc; }
                if let Some(aoc) = entry.talk_anim.and_then(|a| db.try_get_asset_hash(a)) { menu.original_assets[11] = aoc; }
                if let Some(aoc) = entry.demo_anim.and_then(|a| db.try_get_asset_hash(a)) { menu.original_assets[12] = aoc; }
                if let Some(aoc) = entry.hub_anim.and_then(|a| db.try_get_asset_hash(a)) { menu.original_assets[13] = aoc; }
            });
        }
        if photo {
            if menu.preview_data.ubody == 0 && menu.original_assets[0] != 0 {
                menu.preview_data.ubody = menu.original_assets[0];
            }
            if menu.preview_data.uhead == 0 && menu.original_assets[1] != 0 {
                menu.preview_data.uhead = menu.original_assets[1];
            }
            if menu.preview_data.uhair == 0 && menu.original_assets[2] != 0 {
                menu.preview_data.uhair = menu.original_assets[2];
            }
            for x in 0..5 {
                if menu.preview_data.acc[x] == 0 && menu.original_assets[5+x] != 0 {
                    menu.preview_data.acc[x] = menu.original_assets[5+x];
                }
            }
        }
        for x in 0..19 {
            if menu.original_scaling[x] < 10 { menu.original_scaling[x] = 100; }
        }
        for x in 0..32 { menu.color_preview[x] = menu.original_color[x]; }
        for x in 0..16 { menu.scale_preview[x] = menu.original_scaling[x]; }
        modes
    }
}