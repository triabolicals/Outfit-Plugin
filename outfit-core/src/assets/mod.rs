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

pub fn find_aid_condition_prefix(entry: &AssetTable, prefix: &str, with_gender: bool) -> Option<(String, Gender)> {
    let asset_table_sf = AssetTableStaticFields::get();
    let entries = &asset_table_sf.condition_indexes.entries;
    let male  = AssetTableStaticFields::get_condition_index("男装");
    let female  = AssetTableStaticFields::get_condition_index("女装");
    let gender =
        if with_gender{
            if entry.condition_indexes.list.iter().any(|i| i.iter().any(|i| *i == male)) { Some(Gender::Male) }
            else if entry.condition_indexes.list.iter().any(|i| i.iter().any(|i| *i == female)) { Some(Gender::Female) }
            else { None }
        } else { Some(Gender::None) };

    let condition = entry.condition_indexes.list.iter()
        .filter(|i| i.len() == 1)
        .find_map(|l| entries.iter().find(|e| l[0] == e.value && e.key.is_some_and(|a|a.to_string().starts_with(prefix))))
        .map(|s| s.key.unwrap().to_string());

    if gender.is_none() { condition.clone().as_ref().and_then(|c| condition.zip(get_gender_from_condition(c))) }
    else { condition.zip(gender) }
}
pub fn get_gender_from_condition(condition: &String) -> Option<Gender> {
    if condition.starts_with("GID_") {
        GodData::get(condition).map(|v| if v.female == 1 { Gender::Female } else { Gender::Male })
    }
    else if condition.starts_with("PID") {
        PersonData::get(condition).filter(|p| p.parent.index > 1 && p.flag.value & 128 == 0).map(|v| if v.gender == 2 { Gender::Female } else { Gender::Male })
    }
    else if condition.starts_with("MPID_") {
        PersonData::get_list().unwrap().iter().find(|v| v.name.is_some_and(|v| v.to_string() == *condition) && v.gender > 0)
            .map(|v| if v.gender == 2 { Gender::Female } else { Gender::Male })
    }
    else if condition.starts_with("AID_") {
        PersonData::get_list().unwrap().iter().find(|v| v.aid.is_some_and(|v| v.to_string() == *condition) && v.gender > 0)
            .map(|v| if v.gender == 2 { Gender::Female } else { Gender::Male })
    }
    else { None }
}

pub fn get_aid_condition(asset_table_indexes: Vec<i32>, with_gender: bool,) -> Option<(String, Gender)> {
    let s: Vec<_> = asset_table_indexes.into_iter().flat_map(|v| AssetTable::try_index_get(v)).collect();
    if let Some(s) = s.iter().find_map(|x| find_aid_condition_prefix(x, "EID_", with_gender)) {
        return Some(s);
    }
    for prefix in ["EID_", "AID_", "GID_", "MPID_", "PID_", "JID_"]{
        let s = s.iter().find_map(|x| find_aid_condition_prefix(x, prefix, with_gender));
        if s.as_ref().is_some_and(|s| get_condition_label(&s.0).is_some()) {
            return s;
        }
    }
    None
}
pub fn get_condition_label(label: &String) -> Option<String> {
    if let Some(pos) = ["EID_", "AID_", "GID_", "MPID_", "PID_", "JID_"].iter().position(|x| label.starts_with(x)){
        match pos {
            0|2 => {  GodData::get(label.replace("EID_", "GID_")).map(|v| v.mid.to_string()) }
            1 => {
                if let Some(acc) = AccessoryData::get(label.as_str()) { Some(acc.name.to_string()) }
                else if let Some(person) = PersonData::get_list().unwrap().iter().find(|p| p.name.is_some() && p.aid.is_some_and(|s| s.to_string() == *label)){
                    person.name.map(|v| v.to_string())
                }
                else { None }
            }
            3 => Some(label.clone()),
            4 => PersonData::get(label.as_str()).filter(|p| p.parent.index > 1 && p.belong.is_none()).and_then(|p| p.name).map(|v| v.to_string()),
            _ => JobData::get(label.as_str()).map(|j| j.name.to_string()),
        }
    }
    else { None }
}
pub fn get_name_condition(entry: &AssetTable, ) -> Option<String>  {
    let asset_table_sf = AssetTableStaticFields::get();
    let entries = &asset_table_sf.condition_indexes.entries;
    entry.condition_indexes.list.iter()
        .filter(|l| l.len() == 1)
        .find_map(|l| entries.iter()
            .find(|e| l[0] == e.value && e.key.is_some_and(|a| a.str_contains("PID") || a.str_contains("GID") )))
        .map(|s| s.key.unwrap().to_string())
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

pub fn find_mode_1_body(condition_index: i32, gender: Gender) -> Option<String> {
    let asset_table_sf = AssetTableStaticFields::get();
    let gender = if gender == Gender::Female { AssetTableStaticFields::get_condition_index("女装") }
    else { AssetTableStaticFields::get_condition_index("男装") };
    asset_table_sf.search_lists[1].iter().find(|a|{
        a.condition_indexes.has_condition_index(condition_index) && a.body_model.is_some() &&
            a.condition_indexes.has_condition_index(gender)
    })?.body_model.map(|v| v.to_string())
}
pub fn find_mode_1_hair(condition_index: i32) -> Option<String> {
    let asset_table_sf = AssetTableStaticFields::get();
    asset_table_sf.search_lists[1].iter().find(|a|{ a.condition_indexes.has_condition_index(condition_index) && a.head_model.is_some() })?.head_model.map(|v| v.to_string())
}
#[skyline::from_offset(0x1bb4fa0)]
fn result_get_hash_code(this: &AssetTableResult, optional_method: OptionalMethod) -> i32;