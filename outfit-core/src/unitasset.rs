use std::{cmp::PartialEq, fs::{read_dir, read_to_string}};
use engage::{
    app::{
        AssetTable_Modes, AssetTable_Result,
        IAssetTable, IAssetTableMethods, IAssetTable_AccessoryMethods,
        IAssetTable_ConditionIndexesMethods, IAssetTable_Result, IAssetTable_ResultMethods,
        IBitField32, IGameUserDataMethods, IPersonDataMethods, ISingletonClass_1Methods, IStructBase, IStructData_1Methods, IUnit, IUnitEdit, IUnitMethods
    },
    List_1Ext,
    system::collections::generic::IList_1,
    app::{IGodDataMethods, IMapMindMethods, ISortieSelectionUnitManager}
};
use unity::Cast;
pub use crate::playerdata::*;
use crate::{assets::unit_dress_gender, get_outfit_data, AssetConditions, AssetType, Mount, PhotoCameraControl, data::{
    room::hub_room_set_by_result,
    unitselect::{UnitSelect, UnitSelectList}
}, anim::AnimData, room::ReloadType, get_result_color_u8, set_result_scale_u16, set_color_by_u8_slice, set_color_by_i32, il2str, get_result_scale_u16, try_get_il2cpp_hash};

mod load;
pub use load::*;

pub static mut UNIT_ASSET: UnitAssetMenuData = UnitAssetMenuData::default();

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
    pub color_preview: [u8; 64],
    pub scale_preview: [u16; 20],
    pub original_color: [u8; 64],
    pub eye_color: [u8; 18],
    pub original_assets: [i32; 20],
    pub update_dress_gender: bool,
    pub update: u8,
    pub has_head_acc: bool,
    pub has_hair_acc: bool,
    pub preview_asset: Option<(AssetType, i32)>
}
impl UnitAssetPreview {
    pub const fn new() -> Self {
        Self {
            person: 0, gender: 0,
            preview_data:
            PlayerOutfitData::new(),
            selected_profile: 0,
            original_color: [0; 64],
            scale_preview: [0; 20],
            original_scaling: [0; 20],
            color_preview: [0; 64],
            eye_color: [0; 18],
            original_assets: [0; 20],
            update_dress_gender: false,
            update: 0,
            has_head_acc: false,
            preview_asset: None,
            has_hair_acc: false,
        }
    }
    pub fn get_original_asset_hash(&self, asset_type: AssetType) -> i32 {
        match asset_type {
            AssetType::Body => self.original_assets[0],
            AssetType::Head => self.original_assets[1],
            AssetType::Hair => self.original_assets[2],
            AssetType::AOC(k) =>self.original_assets[10+k as usize],
            AssetType::Acc(k) => self.original_assets[5+k as usize],
            AssetType::Voice => self.original_assets[14],
            AssetType::Rig => self.original_assets[15],
            _ => 0,
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
    pub reload_type: Option<ReloadPreview>,
    pub reload_delay: bool,
    pub control: PhotoCameraControl,
    pub photo_profiles: Vec<PlayerOutfitData>,
    pub unit_select: UnitSelectList,
    pub unit_select_index: i32,
}
pub enum LoadResult {
    Success,
    NoFiles,
    MissingDirectory,
}
#[derive(PartialEq, Copy, Clone)]
pub enum ReloadPreview {
    Scale,
    ScalePreview(i32),
    Color(i32),
    ResetColor(i32),
    Preset(usize),
    Asset,
    Full,
    LoadedData,
    Forced,
}
impl UnitAssetMenuData {
    pub fn is_photo_graph() -> bool { Self::get().mode == MenuMode::PhotoGraph }
    pub fn is_unit_info() -> bool { Self::get().mode == MenuMode::UnitInfo }
    pub fn is_shop() -> bool { Self::get().mode == MenuMode::Shop }
    pub fn add_data(&mut self, data: UnitAssetData) {
        if self.data.iter().find(|v| v.person == data.person).is_none() { self.data.push(data); }
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
    pub fn get_result() -> AssetTable_Result {
        let data = Self::get();
        let result =
        match data.mode {
            MenuMode::Shop => {
                data.unit_select.get_result(!data.is_shop_combat)
            }
            _ => {
                UnitSelect {
                    hash: data.preview.person,
                    god: data.god_mode,
                    recruited: false,
                    female: data.preview.gender == 2
                }.get_result(!data.is_shop_combat)
            }
        };
        if data.mode != MenuMode::PhotoGraph {
            AnimData::remove(result, true, true);
            result.set_body_anim(result.m_hub_anim());
            result.set_m_demo_anim(unity::Il2CppString::null());
            result.set_m_talk_anim(unity::Il2CppString::null());
            result.set_m_hub_anim(unity::Il2CppString::null());
            result.set_left_hand("null");
            result.set_right_hand("null");
            result.replace(AssetTable_Modes::combat());
        }
        result
    }
    pub fn get_current_dress_gender() -> i32 { Self::get_preview().gender }
    pub fn get_gender(alt: bool) -> i32 {
        let gender = Self::get_current_dress_gender();
        if !alt { gender } else if gender == 2 { 1 } else { 2 }
    }
    pub fn get() -> &'static mut UnitAssetMenuData { unsafe { &mut UNIT_ASSET } }
    pub fn get_unit() -> Option<engage::app::Unit>{
        if Self::is_shop() {
            let person = engage::app::PersonData::try_get_from_hash(Self::get().preview.person);
            if !person.is_null() {
                let unit = engage::app::UnitPool::get_from_person(person, false);
                if !unit.is_null() { Some(unit) } else { None }
            }
            else { None }
        }
        else {
            let map_mind = engage::app::MapMind::get_instance();
            if !map_mind.is_null() { Some(map_mind.get_unit()) }
            else{
                let sortie = engage::app::SortieSelectionUnitManager::get_instance();
                if !sortie.is_null() {
                    let unit = sortie.m_unit();
                    if !unit.is_null() { Some(unit) } else { None }
                }
                else { None }
            }
        }
    }
    pub fn get_current_profile(hash: i32) -> Option<&'static PlayerOutfitData> {
        Self::get_by_person_data(hash, false)
            .and_then(|p| p.profile.get(p.profile_index(false) as usize))
    }
    const fn default() -> Self {
        Self {
            unit_select_index: 0,
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
            reload_type: None,
            reload_delay: false,
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
    pub fn get_unit_data(unit: engage::app::Unit) -> Option<&'static UnitAssetData>  {
        let person = unit.get_person();
        let hash = person.hash();
        Self::get_by_person_data(hash, false)
            .or_else(||
                if person.is_hero() || ((1  << unit.get_force_type().value) & 25 != 0 && !unit.is_summon() && !unit.is_vision()){
                    Self::get_by_person_data(hash, true)
                }
                else { None }
            )
    }
    pub fn set_god(god: engage::app::GodData){ Self::set_by_hash(god.hash()); }
    pub fn set_by_hash(person: i32) -> bool {
        let menu = Self::get();
        let mut engaged = false;
        let gender;
        let photo = menu.mode == MenuMode::PhotoGraph;
        let person_data = engage::app::PersonData::try_get_from_hash(person);
        if !person_data.is_null() {
            menu.god_mode = false;
            let unit = engage::app::UnitPool::get_from_person(person_data, false);
            if !unit.is_null(){
                engaged = unit.is_engaging_2();
                gender = unit.get_dress_gender().value;
                if unit.get_person().index() == 1 {
                    if let Some(data) = UnitAssetMenuData::get().data.iter_mut().find(|v| v.person == person) {
                        if unit.m_edit().m_gender().value == 2 { data.flag |= 16; }
                    }
                }
            }
            else {
                gender =
                    if person_data.get_flag().m_value() & 32 != 0 { if person_data.get_gender().value == 2 { 1 } else { 2 } }
                    else { if person_data.get_gender().value == 2 { 2 } else { 1 } };
            }
        }
        else {
            let god = engage::app::GodData::try_get_from_hash(person);
            if !god.is_null() {
                let female = god.get_female() as i32;
                menu.god_mode = true;
                gender =
                    if god.is_hero() {
                        let hero = engage::app::UnitPool::get_hero(false);
                        unit_dress_gender(hero)
                    } else { female + 1 };
            }
            else { return false; }
        }
        let s = engage::app::GameUserData::get_instance().get_sequence().value;
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
            let p1 = engage::app::PersonData::try_get_from_hash(person);
            let p2 = engage::app::GodData::try_get_from_hash(person);
            if !p1.is_null() || p2.is_null() {
                if let Some(data) = Self::get_by_person_data(person, true) {
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
            let c = get_result_color_u8(result, x);
            for i in 0..3 { menu.preview.color_preview[x * 4+i] = c[i]; }
        }
        for x in 8..16 {
            for y in 0..3 { menu.preview.color_preview[x*4+y] = menu.preview.preview_data.colors[x].values[y] }
        }
        for x in 0..16 {
            let v = menu.preview.preview_data.scale[x] & 1023;
            if v == 0 || v >= 1000 { menu.preview.scale_preview[x] = get_result_scale_u16(result, x); }
            else { menu.preview.scale_preview[x] = v; }
        }
        menu.preview.update = 3;
        if !photo { hub_room_set_by_result(Some(result), ReloadType::ForcedUpdate); }
        true
    }
    pub fn set_unit(unit: engage::app::Unit) -> bool {
        if unit.is_null() || unit.get_person().is_null() { false }
        else { Self::set_by_hash(unit.get_person().hash()) }
    }
    pub fn get_shop_unit() -> Option<engage::app::Unit> {
        let data = Self::get();
        if data.god_mode { None }
        else {
            crate::utils::person_map(
                engage::app::PersonData::try_get_from_hash(data.preview.person),
                |p| engage::app::UnitPool::get_from_person(p, false)
            )
        }
    }
    pub fn set_reload(kind: ReloadPreview, delay: bool) {
        let data = Self::get();
        data.reload_type = Some(kind);
        data.reload_delay = delay;
    }
    pub fn reload_unit(kind: ReloadPreview) {
        let data = Self::get();
        let result = Self::get_result();
        match kind {
            ReloadPreview::Asset => { return; }
            ReloadPreview::Color(kind) => {
                let mut color: i32 = 0;
                let k = kind as usize;
                if k < 8 {
                    for x in 0..3 { color += data.preview.color_preview[4*kind as usize + x] as i32; }
                    if color > 0 { set_color_by_i32(result, k, color); }
                }
                hub_room_set_by_result(Some(result), ReloadType::ColorScale);
            }
            ReloadPreview::ResetColor(kind) => {
                let k = (kind % 16) as usize;
                let c = [data.preview.original_color[4*k], data.preview.original_color[4*k+1], data.preview.original_color[4*k+2], 255];
                set_color_by_u8_slice(result, k, c);
                hub_room_set_by_result(Some(result), ReloadType::ColorScale);
            }
            ReloadPreview::Scale => { hub_room_set_by_result(Some(result), ReloadType::Scale); }
            ReloadPreview::ScalePreview(kind) => {
                set_result_scale_u16(result, kind as usize,data.preview.scale_preview[kind as usize]);
                hub_room_set_by_result(Some(result), ReloadType::Scale);
            }
            ReloadPreview::Preset(index) => {
                let db = get_outfit_data();
                if let Some(appearance) = db.dress.personal.get(index) {
                    appearance.apply_appearance(result, 2, false, None, &db.hashes, true);
                    result.set_ride_model(unity::Il2CppString::null());
                    result.set_ride_dress_model(unity::Il2CppString::null());
                    result.set_left_hand("null");
                    result.set_right_hand("null");
                    result.set_body_anim(if db.get_dress_gender(result.get_dress_model()) == engage::app::Gender::male() { "AOC_Hub_Hum0M" } else { "AOC_Hub_Hum0F" });
                    hub_room_set_by_result(Some(result), ReloadType::ForcedUpdate);
                }
            }
            ReloadPreview::LoadedData => {
                println!("Loaded Data Reload");
                if let Some(loaded) = data.loaded_data.selected_index.and_then(|i| data.loaded_data.loaded_data.get_mut(i as usize)) {
                    let flag = loaded.data.flag;
                    loaded.data.flag |= 193;
                    loaded.data.set_result(result, 2, false, false);
                    println!("Result Set");
                    loaded.data.flag = flag;
                }
                hub_room_set_by_result(Some(result), ReloadType::ForcedUpdate);
            }
            ReloadPreview::Forced => { hub_room_set_by_result(Some(result), ReloadType::ForcedUpdate); }
            ReloadPreview::Full => { hub_room_set_by_result(Some(result), ReloadType::All); }
        }
        data.reload_type = None;
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
            if let Some(data) = menu.data.iter_mut().find(|x| x.person == hash) {
                if let Some(d) = data.profile.get_mut(index) { *d = new_data.clone(); }
            }
            if menu.god_mode {
                let male = preview.preview_data.mount[0];
                let female = preview.preview_data.mount[1];
                if male != 0 || female != 0 {
                    if let Some(data) = menu.data.iter_mut().find(|x| x.person == hash) {
                        data.profile.iter_mut().for_each(|x| {
                            x.mount[0] = male;
                            x.mount[1] = female;
                        });
                    }
                }
            }
        }
    }
    pub fn get_person_flag() -> i32 {
        let hash = Self::get_preview().person;
        Self::get().data.iter().find(|x| x.person == hash ).map(|x| x.flag).unwrap_or(0)
    }
    pub fn set_assets(result: AssetTable_Result, unit: engage::app::Unit, asset_conditions: &AssetConditions) {
        if il2str(result.get_body_model()).is_some_and(|v| v.contains("AT")) { return; }
        if il2str(result.get_dress_model()).is_some_and(|v| v.contains("AT")) { return; }
        let mode = asset_conditions.mode;
        let menu = Self::get();
        let is_preview =  menu.is_preview;
        let is_engaged = unit.is_engaging_2();
        let is_photo = menu.mode == MenuMode::PhotoGraph;
        let person_hash = unit.get_person().hash();
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
            if let Some(data) = menu.photo_profiles.iter().find(|x| x.break_body == person_hash) {
                data.set_result(result, 2, false, false);
            }
        }
        else if let Some(data) = menu.data.iter().find(|s| s.person == person_hash){
            data.set_result(result, mode, is_engaged, asset_conditions.broken);
        }
    }
    pub fn set_god_assets(result: AssetTable_Result, mode: i32, god: engage::app::GodData, darkness: bool) {
        let menu = Self::get();
        let hash = god.hash();
        if UnitAssetMenuData::is_photo_graph()  {
            if menu.is_preview {
                menu.preview.preview_data.set_result(result, 2, darkness, false);
            }
            else if let Some(data) = menu.photo_profiles.iter().find(|x| x.break_body == hash) {
                data.set_result(result, 2, darkness, false);
            }
            return;
        }
        if let Some(data) = menu.data.iter().find(|s| s.person == hash){
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
    pub fn get_original_color_str(color_index: i32) -> String {
        let k = color_index % 16;
        let i =  4*k as usize;
        let menu = &Self::get_preview().original_color;
        format!("{}/{}/{}", menu[i], menu[i+1], menu[i+2])
    }
    pub fn get_preview_color_str(color_index: i32) -> String {
        let k = color_index % 16;
        let i =  4*k as usize;
        let menu = &Self::get_preview().color_preview;
        format!("{}/{}/{}", menu[i], menu[i+1], menu[i+2])
    }
    pub fn get_set_color_str(color_index: i32) -> String {
        let k = color_index % 16;
        let menu = &Self::get_preview().preview_data.colors[k as usize];
        format!("{}/{}/{}", menu.values[0], menu.values[1], menu.values[2])
    }
    pub fn set_original_assets() -> (Vec<i32>, Vec<i32>){
        let search_lists = engage::app::AssetTable::s_search_lists();
        let flags = engage::app::AssetTable::s_condition_flags();
        let db = get_outfit_data();
        let menu = Self::get_preview();
        let mut modes: (Vec<i32>, Vec<i32>) = (vec![], vec![]);
        menu.original_assets.iter_mut().for_each(|a| {*a = 0});
        menu.original_scaling.iter_mut().for_each(|s| {*s = 0});
        menu.original_color.iter_mut().for_each(|c| {*c = 0});
        menu.original_assets[16] = -1;
        let photo = UnitAssetMenuData::is_photo_graph();
        for mode in 1..3 {
            search_lists.get(mode).iter().filter(|a| a.m_condition_indexes().test(flags)).for_each(|e|{
                let idx = e.index();
                if mode == 1 {
                    modes.0.push(idx);
                    if let Some(h) = try_get_il2cpp_hash(e.get_body_model()).filter(|v| db.hashes.o_body.contains_key(v)) { menu.original_assets[3] = h; }
                    if let Some(h) = try_get_il2cpp_hash(e.get_head_model()).filter(|v| db.hashes.o_hair.contains_key(v)) { menu.original_assets[4] = h; }
                }
                else if mode == 2 {
                    modes.1.push(idx);
                    if let Some(hash) = try_get_il2cpp_hash(e.get_dress_model()).filter(|h| db.hashes.body.contains_key(h)) { menu.original_assets[0] = hash; }
                    if let Some(head) = try_get_il2cpp_hash(e.get_head_model()).filter(|h| db.hashes.head.contains_key(h)) { menu.original_assets[1] = head; }
                    if let Some(hair) = try_get_il2cpp_hash(e.get_hair_model()).filter(|h| db.hashes.hair.contains_key(h)) { menu.original_assets[2] = hair; }
                    if let Some(hair) = e.get_accessories().items().iter()
                        .filter(|x| !x.is_null())
                        .find(|x| il2str(x.get_model()).is_some_and(|v| v.contains("Hair")))
                        .and_then(|x| try_get_il2cpp_hash(x.get_model()))
                        .filter(|hash| db.hashes.hair.contains_key(hash))
                    {
                        menu.original_assets[2] = hair;
                    }
                    for xx in 0..5 {
                        if let Some(acc) =
                            e.get_accessories().items().iter()
                                .filter(|x| !x.is_null())
                                .find(|x| il2str(x.get_locator()).is_some_and(|v| v == ACC_LOC[xx]) && !x.get_model().is_null())
                                .and_then(|x| try_get_il2cpp_hash(x.get_model()))
                                .filter(|hash| db.hashes.hair.contains_key(hash))
                        {
                            menu.original_assets[5 + xx] = acc;
                        }
                    }
                    if let Some(rig) = try_get_il2cpp_hash(e.get_body_model()).filter(|g| db.hashes.rigs.contains_key(g)) { menu.original_assets[15] = rig; }
                }
                if let Some(ride) = il2str(e.get_ride_dress_model()){
                    let mount = Mount::from(ride.as_str()) as i32 - 1;
                    if menu.original_assets[16] < 0 && mount >= 0 { menu.original_assets[16] = mount; }
                }
                for x in 0..19 {
                    let v = crate::get_asset_table_scale_u16(e, x);
                    if v < 10 && v >= 1000 { menu.original_scaling[x] = 100; } else { menu.original_scaling[x] = v; }
                }
                for x in 0..8 {
                    let color = crate::get_asset_table_color_u8_slice(e, x);
                    for i in 0..3 { menu.original_color[4*x+i] = color[i]; }
                }
                if let Some(h) = try_get_il2cpp_hash(e.get_info_anim()).filter(|h| db.hashes.aoc.contains_key(h)) { menu.original_assets[10] = h; }
                if let Some(h) = try_get_il2cpp_hash(e.get_talk_anim()).filter(|h| db.hashes.aoc.contains_key(h)) { menu.original_assets[11] = h; }
                if let Some(h) = try_get_il2cpp_hash(e.get_demo_anim()).filter(|h| db.hashes.aoc.contains_key(h)) { menu.original_assets[12] = h; }
                if let Some(h) = try_get_il2cpp_hash(e.get_hub_anim()).filter(|h| db.hashes.aoc.contains_key(h)) { menu.original_assets[13] = h; }
                if let Some(h) = try_get_il2cpp_hash(e.get_voice()).filter(|h| db.hashes.voice.contains_key(h)) { menu.original_assets[14] = h; }
            });
        }
        if photo {
            if menu.preview_data.ubody == 0 && menu.original_assets[0] != 0 { menu.preview_data.ubody = menu.original_assets[0]; }
            if menu.preview_data.uhead == 0 && menu.original_assets[1] != 0 { menu.preview_data.uhead = menu.original_assets[1]; }
            if menu.preview_data.uhair == 0 && menu.original_assets[2] != 0 { menu.preview_data.uhair = menu.original_assets[2]; }
            for x in 0..5 {
                if menu.preview_data.acc[x] == 0 && menu.original_assets[5+x] != 0 { menu.preview_data.acc[x] = menu.original_assets[5+x]; }
            }
        }
        for x in 0..19 { if menu.original_scaling[x] < 10 { menu.original_scaling[x] = 100; } }
        for x in 0..32 { menu.color_preview[x] = menu.original_color[x]; }
        for x in 0..16 { menu.scale_preview[x] = menu.original_scaling[x]; }
        modes
    }
}