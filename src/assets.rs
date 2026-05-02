use unity::prelude::*;
use engage::{unit::*, gamedata::{assettable::*, item::ItemData, skill::*, *}, };
pub mod transform;
pub mod dress;

use outfit_core::*;
use outfit_core::room::CharacterEffect;
// use crate::assets::transform::is_dragonstone;
use crate::enums::PIDS;

#[skyline::hook(offset=0x1bb4180)]
pub fn asset_table_setup_person_outfit(
    this: &mut AssetTableResult,
    mode: i32,
    person: Option<&PersonData>,
    conditions: &mut Array<&'static Il2CppString>,
    method_info: OptionalMethod) -> &'static mut AssetTableResult
{
    let result = call_original!(this, mode, person, conditions, method_info);
    if is_tiki_engage(result) { return result;}
    if let Some(v) = person.and_then(|p| UnitAssetMenuData::get_by_person_data(p.parent.hash, false)){
        v.set_result(result, mode, false, false);
    }
    result
}
#[skyline::hook(offset=0x01bb2430)]
pub fn asset_table_result_setup_hook_outfit(
    this: &mut AssetTableResult,
    mode: i32,
    unit: &mut Unit,
    equipped: Option<&ItemData>,
    conds: &mut Array<&'static Il2CppString>,
    method_info: OptionalMethod
) -> &'static mut AssetTableResult
{
    let result = call_original!(this, mode, unit, equipped, conds, method_info);
    let mut conditions = AssetConditions::new(None, mode, equipped);

    dress::commit_for_unit_dress(result, mode, unit, equipped, conds, &mut conditions);
    result
}

#[skyline::hook(offset=0x01bb2d80)]
pub fn asset_table_result_god_setup_outfit(
    this: &mut AssetTableResult,
    mode: i32,
    god_data: Option<&GodData>,
    is_darkness: bool,
    conditions: &mut Array<&'static Il2CppString>,
    method_info: OptionalMethod
) -> &'static mut AssetTableResult
{
    let result = call_original!(this, mode, god_data, is_darkness, conditions, method_info);
    if let Some(god) = god_data {
        let menu_data = UnitAssetMenuData::get();
        if menu_data.is_preview { menu_data.preview.preview_data.set_result(result, 2, is_darkness, false); }
        else { UnitAssetMenuData::set_god_assets(result, mode, god, is_darkness); }
    }
    result
}
pub fn unit_dress_gender(unit: &Unit) -> i32 {
    if unit.person.pid.to_string() == PIDS[0] || unit.person.flag.value & 128 != 0 {  if unit.edit.is_enabled() { return unit.edit.gender; }  }
    unit.person.get_dress_gender() as i32
}

pub fn is_sword_fighter_outfit(this: &mut AssetTableResult) -> bool {
    if !this.dress_model.is_null() {
        let dress_model = this.dress_model.to_string();
        dress_model.contains("Swd0A") && !dress_model.contains("c251")
    }
    else if !this.body_model.is_null() {
        let body_model = this.body_model.to_string();
        body_model.contains("Swd0A")  && !body_model.contains("c251")
    }
    else { false }
}

pub fn is_tiki_engage(this: &mut AssetTableResult) -> bool {
    if !this.dress_model.is_null() { this.dress_model.to_string().contains("Tik1AT") }
    else if !this.body_model.is_null() { this.body_model.to_string().contains("Tik1AT") }
    else { false }
}

pub fn is_monster_body(this: &mut AssetTableResult) -> bool {
    if !this.dress_model.is_null() { this.dress_model.to_string().contains("T_c") }
    else if !this.body_model.is_null() { this.body_model.to_string().contains("T_c") }
    else { false }
}

#[unity::hook("Combat", "CharacterEffect", "CreateBreak")]
pub fn create_break_effect_hook(this: &mut CharacterEffect, method_info: OptionalMethod) {
    call_original!(this, method_info);
    outfit_core::room::break_effect(this);
}
