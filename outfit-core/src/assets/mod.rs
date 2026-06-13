use std::collections::HashMap;
use std::num::Wrapping;
use engage::{
    unit::{Unit, Gender},
    gamedata::{accessory::AccessoryData, Gamedata, GodData, JobData, PersonData}
};
use engage_il2cpp::app::{AssetTable, IAssetTable, IAssetTableMethods, IAssetTable_ConditionIndexes, IStructBase, IStructData_1Methods};
use engage_il2cpp::List_1Ext;
use engage_il2cpp::system::collections::generic::IDictionary_2Methods;
use unity2::Cast;
pub use unity::prelude::*;
mod accessory;
mod conditions;
mod result;

pub use accessory::*;
pub use result::*;
pub use conditions::{AssetFlags, AssetConditions, CharacterAssetMode};

pub fn find_aid_condition_prefix(entry: AssetTable, prefix: &str, with_gender: bool, map: &HashMap<i32, String>) -> Option<(String, Gender)> {
    let male = AssetTable::s_condition_indexes().get_item("男装".into());
    let female = AssetTable::s_condition_indexes().get_item("女装".into());
    let entry_indexes = entry.m_condition_indexes();
    let gender =
        if with_gender{
            if entry_indexes.m_list().iter().any(|i| i.iter().any(|i| *i == male)) { Some(Gender::Male) }
            else if entry_indexes.m_list().iter().any(|i| i.iter().any(|i| *i == female)) { Some(Gender::Female) }
            else { None }
        }
        else { Some(Gender::None) };

    let condition = entry_indexes.m_list().iter()
        .filter(|i| i.iter().len() == 1)
        .find_map(|i| i.iter().find(|idx| map.get(&idx).is_some_and(|v| v.starts_with(prefix))))
        .and_then(|i| map.get(&i).cloned());

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

pub fn get_aid_condition(asset_table_indexes: Vec<i32>, with_gender: bool, map: &HashMap<i32, String>) -> Option<(String, Gender)> {
    let s: Vec<_> = asset_table_indexes.into_iter()
        .flat_map(|v| {
            let e = AssetTable::try_get_2(*v);
            if e.is_null() { None } else { Some(e) }
        }).collect();
    if let Some(s) = s.iter().find_map(|x| find_aid_condition_prefix(*x, "EID_", with_gender, map)) {
        return Some(s);
    }
    for prefix in ["EID_", "AID_", "GID_", "MPID_", "PID_", "JID_"]{
        let s = s.iter().find_map(|x| find_aid_condition_prefix(*x, prefix, with_gender, map));
        if s.as_ref().is_some_and(|s| get_condition_label(&s.0).is_some()) {
            return s;
        }
    }
    None
}
pub fn get_condition_index(condition: impl Into<unity2::Il2CppString>) -> Option<i32> {
    let (found, idx) =
    AssetTable::s_condition_indexes().try_get_value(condition.into());
    if found { Some(idx) } else { None }
}
pub fn has_condition_index(entry: AssetTable, condition_index: i32) -> bool {
    entry.m_condition_indexes().m_list().iter().any(|i| i.iter().any(|i| *i == condition_index))
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

pub fn find_entries_with_model_field(mode: i32, model: &str, filter: impl Fn(AssetTable, &str) -> bool ) -> Vec<i32> {
    AssetTable::s_search_lists().get(mode as usize).iter().filter(|e| filter(*e, model)).map(|e| e.index()).collect()
}

pub fn find_mode_1_body(condition_index: i32, gender: Gender) -> Option<String> {
    let gender = if gender == Gender::Female { AssetTable::s_condition_indexes().get_item("女装".into()); }
    else { AssetTable::s_condition_indexes().get_item("男装".into()); };
    AssetTable::s_search_lists().get(1).iter().find(|a|{
        let con_idx = a.m_condition_indexes();
        let condition_match = con_idx.m_list().iter().flat_map(|i| i.iter()).any(|i| *i == condition_index);
        let gender_match = con_idx.m_list().iter().flat_map(|i| i.iter()).any(|i| *i == gender);
        condition_match && gender_match && !a.get_body_model().is_null()
    }).map(|v| v.get_body_model().to_rust_string())
}
pub fn find_mode_1_hair(condition_index: i32) -> Option<String> {
    AssetTable::s_search_lists().get(1).iter().find(|a|{
        let con_idx = a.m_condition_indexes();
        let condition_match = con_idx.m_list().iter().flat_map(|i| i.iter()).any(|i| *i == condition_index);
        condition_match && !a.get_hair_model().is_null()
    }).map(|v| v.get_hair_model().to_rust_string())
}
#[skyline::from_offset(0x1bb4fa0)]
fn result_get_hash_code(this: &AssetTableResult, optional_method: OptionalMethod) -> i32;