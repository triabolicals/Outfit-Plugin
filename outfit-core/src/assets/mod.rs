use std::num::Wrapping;
use std::ops::Add;
use engage::{
    unit::{Unit, Gender},
    gamedata::{assettable::*, accessory::AccessoryData, Gamedata, GodData, JobData, PersonData}
};
pub use unity::prelude::*;
mod accessory;
mod conditions;

pub use accessory::*;
pub use conditions::{AssetFlags, AssetConditions, CharacterAssetMode};
pub fn search_by_2_keys<'a>(mode: i32, key1: impl Into<&'static Il2CppString>,  key2: impl Into<&'static Il2CppString>) -> Option<&'static &'static mut AssetTable> {
    let asset_table_sf = AssetTableStaticFields::get();
    let key1_index  = AssetTableStaticFields::get_condition_index(key1);
    let key2_index  = AssetTableStaticFields::get_condition_index(key2);
    if key1_index < 1 || key2_index < 1 { return None; }
    asset_table_sf.search_lists[mode as usize].iter().find(|entry|
        entry.mode == mode && has_condition(entry, key1_index) && has_condition(entry, key2_index)
    )
}
pub fn get_aid_condition(entry: &AssetTable) -> Option<(String, Gender)> {
    let asset_table_sf = AssetTableStaticFields::get();
    let entries = &asset_table_sf.condition_indexes.entries;
    let male  = AssetTableStaticFields::get_condition_index("男装");
    let female  = AssetTableStaticFields::get_condition_index("女装");

    let gender = if entry.condition_indexes.list.iter().any(|i| i.iter().any(|i| *i == male)) { Gender::Male }
    else if entry.condition_indexes.list.iter().any(|i| i.iter().any(|i| *i == female)) { Gender::Female }
    else { Gender::None };
    let ss =
        entry.condition_indexes.list.iter()
            .filter(|i| i.len() == 1)
            .find_map(|l|
                entries.iter().find(|e| l[0] == e.value &&
                    e.key.is_some_and(|a|{
                        let a_key = a.to_string();
                        a_key.starts_with("AID_") || a_key.starts_with("PID_") || a_key.starts_with("GID_") || a_key.starts_with("JID_")
                    })
                )
            )
            .map(|s| s.key.unwrap().to_string())
            .or_else(|| get_name_condition(entry));
    if gender == Gender::None {
        if let Some(s) = ss.as_ref() {
            let gender =
                if s.contains("AID_") {
                    AccessoryData::get(s.as_str()).map(|a| a.condition_gender)
                        .or_else(|| PersonData::get_list().unwrap().iter()
                            .find(|p| p.aid.is_some_and(|aid| aid.to_string() == *s))
                            .map(|p| p.get_gender() as i32)
                        ).unwrap_or(0)
                }
                else if s.contains("JID_") {
                    JobData::get(s.as_str()).map(|j| {
                        if j.flag.value & 4 != 0 || (j.unit_icon_id_m.is_none() && j.unit_icon_id_f.is_some()) { 2 } else if j.flag.value & 16 != 0 || (j.unit_icon_id_m.is_some() && j.unit_icon_id_f.is_none()) { 1 } else { 0 }
                    }).unwrap_or(0)
                }
                else if s.contains("PID_") { PersonData::get(s.as_str()).map(|p| p.get_gender() as i32).unwrap_or(0) }
                else if s.contains("GID_") { GodData::get(s.as_str()).map(|p| if p.female == 0 { 1 } else { 2 }).unwrap_or(0) }
                else { 0 };
            return
                if gender == 1 {ss.zip(Some(Gender::Male)) }
                else if gender == 2 { return ss.zip(Some(Gender::Female)) }
                else { ss.zip(Some(Gender::None)) };
        }
    }
    ss.zip(Some(gender))
}
pub fn get_name_condition(entry: &AssetTable, ) -> Option<String>  {
    let asset_table_sf = AssetTableStaticFields::get();
    let entries = &asset_table_sf.condition_indexes.entries;
    entry.condition_indexes.list.iter()
        .filter(|l| l.len() == 1)
        .find_map(|l| entries.iter()
            .find(|e| l[0] == e.value && e.key.is_some_and(|a| a.str_contains("PID") || a.str_contains("GID") || a.str_contains("JID"))))
        .map(|s| s.key.unwrap().to_string())
}
pub fn has_condition(entry: &AssetTable, condition_index: i32) -> bool {
    entry.condition_indexes.list.iter().any(|s| s.iter().any(|&index| index ==  condition_index))
}

pub fn new_result_get_hash_code(this: &AssetTableResult, optional_method: OptionalMethod) -> i32 {
    let original = unsafe { result_get_hash_code(this, optional_method) };
    let mut new_hash = Wrapping(original);
    for x in 0..16 {
        let v = (this.scale_stuff[x] * 1000.0) as i32 * x as i32;
        new_hash = new_hash.add(Wrapping(v));
    }
    for x in 0..8 {
        let hash = (this.unity_colors[x].r * 255.0) as i32 + (((this.unity_colors[x].g * 255.0)as i32) << 8) + (((this.unity_colors[x].b * 255.0) as i32) << 16);
        new_hash = new_hash.add(Wrapping(hash));
    }
    new_hash.0
}

pub fn unit_dress_gender(unit: &Unit) -> i32 {
    if unit.edit.is_enabled() { unit.edit.gender }
    else { unit.person.get_dress_gender() as i32 }
}

pub fn find_entries_with_model_field(mode: i32, model: &str, filter: impl Fn(&AssetTable, &str) -> bool ) -> Vec<i32> {
    let asset_table_sf = AssetTableStaticFields::get();
    asset_table_sf.search_lists[mode as usize].iter()
        .filter(|entry| filter(entry, model))
        .map(|entry| entry.parent.index ).collect()
}

#[skyline::from_offset(0x1bb4fa0)]
fn result_get_hash_code(this: &AssetTableResult, optional_method: OptionalMethod) -> i32;