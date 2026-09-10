use engage::{
    prelude::*,
    app::{AssetTable_Result, IAssetTable_ResultMethods, IStructBase}
};
use engage::app::{IGodDataMethods, IStructData_1Methods};
use unity::Cast;
pub mod transform;
pub mod dress;
use outfit_core::*;

#[skyline::hook(offset=0x1bb4180)]
pub fn asset_table_setup_person_outfit(
    this: AssetTable_Result,
    mode: i32,
    person: engage::app::PersonData,
    conditions: Array<Il2CppString>,
    method_info: OptionalMethod) -> AssetTable_Result
{
    let result = call_original!(this, mode, person, conditions, method_info);
    if is_tiki_engage(result) { return result;}
    if !person.is_null() {
        if let Some(v) = UnitAssetMenuData::get_by_person_data(person.hash(), false){
            v.set_result(result, mode, false, false);
        }
    }
    result
}
#[skyline::hook(offset=0x01bb2430)]
pub fn asset_table_result_setup_hook_outfit(
    this: AssetTable_Result,
    mode: i32,
    unit: Unit,
    equipped: engage::app::ItemData,
    conds: Array<Il2CppString>,
    method_info: OptionalMethod
) -> AssetTable_Result
{
    let result = call_original!(this, mode, unit, equipped, conds, method_info);
    let mut conditions = AssetConditions::new(unit, mode, equipped);
    dress::commit_for_unit_dress(result, mode, unit, equipped, conds, &mut conditions);
    result
}

#[skyline::hook(offset= 0x2b0ed80)]
pub fn appearance_create_from_result_outfit(this: AssetTable_Result, map_distance: i32, o: OptionalMethod) -> CharacterAppearance {
    let appearance:  CharacterAppearance = call_original!(this, map_distance, o);
    if !appearance.is_null() {
        if !this.get_pid().is_null() {
            let person = engage::app::PersonData::get(this.get_pid());
            if !person.is_null() { unity::field_set_value_at_offset::<i32>(appearance, 0xd4, person.hash()); }
            else {
                let god = engage::app::GodData::get(this.get_pid());
                if !god.is_null() { unity::field_set_value_at_offset::<i32>(appearance, 0xd4, god.hash()); }
            }
        }
    }
    appearance
}

#[skyline::hook(offset=0x01bb2d80)]
pub fn asset_table_result_god_setup_outfit(
    this: AssetTable_Result,
    mode: i32,
    god_data: engage::app::GodData,
    is_darkness: bool,
    conds: Array<Il2CppString>,
    method_info: OptionalMethod
) -> AssetTable_Result
{
    let result = call_original!(this, mode, god_data, is_darkness, conds, method_info);
    if !god_data.is_null() {
        let menu_data = UnitAssetMenuData::get();
        if menu_data.is_preview { menu_data.preview.preview_data.set_result(result, 2, is_darkness, false); }
        else { UnitAssetMenuData::set_god_assets(result, mode, god_data, is_darkness); }
        result.set_pid(god_data.get_gid());
    }
    result
}

pub fn is_tiki_engage(this: AssetTable_Result) -> bool {
    let dress = this.get_dress_model();
    if !dress.is_null() { if dress.to_rust_string().contains("Tik1AT") { return true; } }
    let dress = this.get_body_model();
    if !dress.is_null() { if dress.to_rust_string().contains("Tik1AT") { return true;} }
    false
}

pub fn is_monster_body(this: AssetTable_Result) -> bool {
    let dress = this.get_dress_model();
    if !dress.is_null() { if dress.to_rust_string().contains("T_c") { return true; } }
    let dress = this.get_body_model();
    if !dress.is_null() { if dress.to_rust_string().contains("T_c") { return true;} }
    false
}