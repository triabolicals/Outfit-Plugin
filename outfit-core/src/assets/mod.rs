use std::collections::HashMap;
use engage::{
    system::collections::generic::IDictionary_2Methods,
    List_1Ext,
    app::{AssetTable, IAccessoryDataMethods, IAssetTable, IAssetTableMethods, IAssetTable_ConditionIndexes, IBitField32, IGodDataMethods, IStructBase, IStructData_1Methods},
    app::IPersonDataMethods,
    app::IJobDataMethods,
    system::collections::generic::IList_1Methods,
    app::{IUnit, IUnitEdit, IUnitMethods}
};
use unity::Cast;
mod accessory;
mod conditions;
mod result;

pub use accessory::*;
pub use result::*;
pub use conditions::{AssetFlags, AssetConditions, CharacterAssetMode};
use crate::il2str;

pub fn find_aid_condition_prefix(entry: AssetTable, prefix: &str, with_gender: bool, map: &HashMap<i32, String>) -> Option<(String, engage::app::Gender)> {
    let male = AssetTable::s_condition_indexes().get_item("男装".into());
    let female = AssetTable::s_condition_indexes().get_item("女装".into());
    let entry_indexes = entry.m_condition_indexes();
    let gender =
        if with_gender{
            if entry_indexes.m_list().iter().any(|i| i.iter().any(|i| i == male)) { Some(engage::app::Gender::male()) }
            else if entry_indexes.m_list().iter().any(|i| i.iter().any(|i| i == female)) { Some(engage::app::Gender::female()) }
            else { None }
        }
        else { Some(engage::app::Gender::none()) };
    let condition = entry_indexes.m_list().iter()
        .filter(|i| i.count() == 1)
        .find_map(|i| i.iter().find(|idx| map.get(&idx).is_some_and(|v| v.starts_with(prefix))))
        .and_then(|i| map.get(&i).cloned());

    if gender.is_none() { condition.clone().as_ref().and_then(|c| condition.zip(get_gender_from_condition(c))) }
    else { condition.zip(gender) }
}
pub fn get_gender_from_condition(condition: &String) -> Option<engage::app::Gender> {
    if condition.starts_with("GID_") {
        let god_data = engage::app::GodData::get(condition.as_str().into());
        if !god_data.is_null() {
            if god_data.get_female() == 1 { Some(engage::app::Gender::female()) }
            else { Some(engage::app::Gender::male()) }
        }
        else { None }
    }
    else if condition.starts_with("PID") {
        let data = engage::app::PersonData::get(condition.as_str().into());
        if !data.is_null() {
            if data.index() > 1 && data.get_flag().m_value() & 128 == 0 {
                let gen = data.get_gender().value;
                if gen == 2 { Some(engage::app::Gender::female()) }
                else if gen == 1 { Some(engage::app::Gender::male()) }
                else { None }
            }
            else { None }
        }
        else { None }
    }
    else if condition.starts_with("MPID_") {
        let person = engage::app::PersonData::get_list();
        if let Some(p) = person.iter().find(|v| il2str(v.get_name()).is_some_and(|v| *v == *condition)) {
            let gender = p.get_gender().value;
            if gender == 1 { Some(engage::app::Gender::male()) }
            else if gender == 2 { Some(engage::app::Gender::female()) }
            else { None }
        }
        else { None }
    }
    else if condition.starts_with("AID_") {
        let person = engage::app::PersonData::get_list();
        if let Some(p) = person.iter().find(|v| il2str(v.get_aid()).is_some_and(|v| *v == *condition)) {
            let gender = p.get_gender().value;
            if gender == 1 { Some(engage::app::Gender::male()) }
            else if gender == 2 { Some(engage::app::Gender::female()) }
            else { None }
        }
        else { None }
    }
    else { None }
}

pub fn get_aid_condition(asset_table_indexes: Vec<i32>, with_gender: bool, map: &HashMap<i32, String>) -> Option<(String, engage::app::Gender)> {
    let list = AssetTable::get_list();
    if let Some(s) = asset_table_indexes.iter()
        .map(|&idx| list.get(idx))
        .find_map(|x| find_aid_condition_prefix(x, "EID_", with_gender, map))
    {
        return Some(s);
    }
    for prefix in ["EID_", "AID_", "GID_", "MPID_", "PID_", "JID_"]{
        if let Some(s) = asset_table_indexes.iter()
            .map(|&idx| list.get(idx))
            .find_map(|x| find_aid_condition_prefix(x, prefix, with_gender, map))
        {
            if get_condition_label(&s.0).is_some() { return Some(s); }
        }
    }
    None
}
pub fn get_condition_index2(condition: unity::Il2CppString) -> Option<i32> {
    if condition.is_null() { None }
    else {
        let (found, idx) =
            AssetTable::s_condition_indexes().try_get_value(condition.into());
        if found { Some(idx) } else { None }
    }

}
pub fn get_condition_index(condition: impl Into<unity::Il2CppString>) -> Option<i32> {
    let (found, idx) =
    AssetTable::s_condition_indexes().try_get_value(condition.into());
    if found { Some(idx) } else { None }
}
pub fn has_condition_index(entry: AssetTable, condition_index: i32) -> bool {
    entry.m_condition_indexes().m_list().iter().any(|i| i.iter().any(|i| i == condition_index))
}
pub fn get_condition_label(label: &String) -> Option<String> {
    if let Some(pos) = ["EID_", "AID_", "GID_", "MPID_", "PID_", "JID_"].iter().position(|x| label.starts_with(x)){
        match pos {
            0|2 => {
                let label = label.replace("EID_", "GID_");
                let god = engage::app::GodData::get(label.as_str().into());
                if !god.is_null() { il2str(god.get_mid()) } else { None }
            }
            1 => {
                let acc = engage::app::AccessoryData::get(label.as_str().into());
                if !acc.is_null() { il2str(acc.get_name()) }
                else {
                    let list = engage::app::PersonData::get_list();
                    list.iter().find(|p|{
                        let name = il2str(p.get_name());
                        let aid = il2str(p.get_aid());
                        name.is_some() && aid.is_some_and(|v| v == *label)
                    }).and_then(|v| il2str(v.get_name()))
                }
            }
            3 => Some(label.clone()),
            4 =>{
                let person = engage::app::PersonData::get(label.as_str().into());
                if !person.is_null() { il2str(person.get_name()) } else { None }
            }
            _ => {
                let job = engage::app::JobData::get(label.as_str().into());
                if !job.is_null() { il2str(job.get_name()) } else { None }
            }
        }
    }
    else { None }
}
pub fn unit_dress_gender(unit: engage::app::Unit) -> i32 {
    if unit.m_edit().m_gender().value != 0 {unit.m_edit().m_gender().value }
    else { unit.get_dress_gender().value }
}

pub fn find_entries_with_model_field(mode: i32, model: &str, filter: impl Fn(AssetTable, &str) -> bool ) -> Vec<i32> {
    AssetTable::s_search_lists().get(mode as usize).iter().filter(|e| filter(*e, model)).map(|e| e.index()).collect()
}

pub fn find_mode_1_body(condition_index: i32, gender: engage::app::Gender) -> Option<String> {
    let gender = if gender == engage::app::Gender::female() { AssetTable::s_condition_indexes().get_item("女装".into()) }
    else { AssetTable::s_condition_indexes().get_item("男装".into()) };
    AssetTable::s_search_lists().get(1).iter().find(|a|{
        let con_idx = a.m_condition_indexes();
        let condition_match = con_idx.m_list().iter().flat_map(|i| i.iter()).any(|i| i == condition_index);
        let gender_match = con_idx.m_list().iter().flat_map(|i| i.iter()).any(|i| i == gender);
        condition_match && gender_match && !a.get_body_model().is_null()
    }).map(|v| v.get_body_model().to_rust_string())
}
pub fn find_mode_1_hair(condition_index: i32) -> Option<String> {
    AssetTable::s_search_lists().get(1).iter().find(|a|{
        let con_idx = a.m_condition_indexes();
        let condition_match = con_idx.m_list().iter().flat_map(|i| i.iter()).any(|i| i == condition_index);
        condition_match && !a.get_head_model().is_null()
    }).map(|v| v.get_head_model().to_rust_string())
}