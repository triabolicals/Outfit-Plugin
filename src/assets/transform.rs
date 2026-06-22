use super::*;
use engage::{
    combat::{CharacterAppearance, CombatRecord, CombatStyle, ICharacterGameStatus, ICharacterGameStatusMethods, ICombatRecordMethods},
    app::{AssetTable_Modes, BattleCalculator, BattleSide_Type, IGodDataMethods, IGodUnit, IItemDataMethods, IJobDataMethods, IPersonDataMethods, IUnitItem, IUnitMethods},
};

pub const MONSTERS: [&str; 8] = ["JID_幻影飛竜", "JID_異形飛竜", "JID_幻影狼", "JID_異形狼",  "JID_E006ラスボス", "JID_幻影竜", "JID_異形竜", "JID_邪竜"];
pub const MONSTER_PERSONS: [&str; 8] = [
    "PID_G000_幻影飛竜", "PID_E004_異形兵_異形飛竜", "PID_G000_幻影狼", "PID_E001_異形兵_異形狼",
    "PID_E006_Boss", "PID_S006_幻影竜", "PID_M019_異形竜", "PID_M026_ソンブル_竜型"
];
pub const SCALE: [f32; 8] = [  1.0, 1.0, 1.0, 1.0, 0.40, 1.0, 1.0, 0.40];

#[repr(C)]
pub struct CombatRecordDisplayClass85 {
    pub this: CombatRecord,
    pub calc: BattleCalculator,
}

#[repr(C)]
pub struct CombatRecordDisplayClass87 {
    pub calc: BattleCalculator,
    pub pre_index: i32,
    pub this: CombatRecord,
}
pub fn is_monster_class(unit: engage::app::Unit) -> bool {
    if unit.get_person().get_bmap_size() > 1 || unit.get_person().get_gender().value == 0 { false }
    else {
        let jid = unit.get_job().get_jid().to_rust_string();
        MONSTERS.contains(&jid.as_str())
    }
}
pub fn has_enemy_tiki(unit: engage::app::Unit) -> bool {
    let god_unit = unit.get_god_unit();
    if !god_unit.is_null() { god_unit.m_data().get_gid().to_rust_string().contains("敵チキ") }
    else { unit.get_pid().to_rust_string().contains("チキ") }
}

pub fn is_tiki_engage(unit: engage::app::Unit) -> bool {
    unit.is_engaging_2() && {
        let god_unit = unit.get_god_unit();
        if !god_unit.is_null() { god_unit.m_data().get_gid().to_rust_string().contains("チキ") } else { false }
    }
}

#[skyline::hook(offset=0x029285f0)]
pub fn change_dragon2(this: CombatRecord, calc_side: BattleSide_Type, param_3: &CombatRecordDisplayClass85, method_info: unity::OptionalMethod) {
    call_original!(this, calc_side, param_3, method_info);
    let side = engage::combat::Side::convert_from(calc_side, this.get_is_enemy_attack());
    if side < 0 { return; }
    let side = side as usize;
    let status_side = this.get_game_status().get(side);
    if status_side.is_null() { return; }
    if status_side.get_unit().is_null() { return; }
    let unit = status_side.get_unit();
    let pid = unit.get_pid().to_rust_string();
    if unit.get_person().get_bmap_size() > 1 || unit.get_person().get_gender().value == 0 ||
        pid.contains("PID_E001_Boss") || pid.contains("PID_E006_Boss") || pid.contains("チキ") {
        return;
    }
    let map_distance = this.get_map_distance();
    let distance = if map_distance < 1 { 1 } else if map_distance > 2 { 2 } else { map_distance };
    if is_tiki_engage(unit) { return; }
    let job = unit.get_job();
    let can_dragon_stone = unit.has_skill_2("SID_竜石装備") && job.get_max_weapon_level(9).value > 1;
    let item = status_side.get_weapon();
    if !item.is_null() {
        let item = item.m_item();
        if item.is_dragon() || (item.get_kind().value == 9 && can_dragon_stone) {
            this.set_combat_style(CombatStyle::dragon_change());
            let dragon = this.get_game_status_dragonize().get(side);
            dragon.import_4(side as i32, param_3.calc, calc_side, distance);
            status_side.set_appearance(CharacterAppearance::create_from_result(get_transform_result(unit), distance));
        }
    }
    let db = get_outfit_data();
    if let Some(monster_data) = db.dress.transform.iter().find(|x| !x.is_transform && job.hash() == x.hash){
        this.set_combat_style(CombatStyle::dragon_change());
        let dragon = this.get_game_status_dragonize().get(side);
        dragon.import_4(side as i32, param_3.calc, calc_side, distance);
        status_side.set_appearance(CharacterAppearance::create_from_result(monster_data.get_result(2, unit), distance));
    }
}

#[skyline::hook(offset=0x02928bc0)]
pub fn transformation_chain_atk(this: CombatRecord, calc_side: i32, param_3: &CombatRecordDisplayClass87, method_info: unity::OptionalMethod) {
    let chain_atk_index = this.get_chain_attack_count();
    call_original!(this, calc_side, param_3, method_info);
    let count = this.get_chain_attack_count();
    if chain_atk_index < count {
        let side = this.get_game_status_chain_atk().get(chain_atk_index as usize);
        if !side.is_null() {
            let unit = side.get_unit();
            if !unit.is_null() {
                let job = unit.get_job();
                let can_dragon_stone = unit.has_skill_2("SID_竜石装備") && job.get_max_weapon_level(9).value > 1;
                let item = side.get_weapon().m_item();
                if is_dragonstone(item) || (item.get_kind().value == 9 && can_dragon_stone) {
                    side.set_appearance(CharacterAppearance::create_from_result(get_transform_result(unit), 1));
                    return;
                }
                let db = get_outfit_data();
                if let Some(monster_data) = db.dress.transform.iter().find(|x| !x.is_transform && job.hash() == x.hash) {
                    side.set_appearance(CharacterAppearance::create_from_result(monster_data.get_result(2, unit), 1));
                }
            }
        }
    }
}
fn get_transform_result(unit: engage::app::Unit) -> AssetTable_Result {
    let db = get_outfit_data();
    let job = unit.get_job().hash();
    db.dress.transform.iter().find(|x| x.is_transform && job == x.hash).map(|data| data.get_result(2, unit))
        .unwrap_or(
            AssetTable_Result::get_from_pid(AssetTable_Modes::combat(),if unit.get_gender().value == 2 { "PID_エル_竜化" } else { "PID_ラファール_竜化"}, CharacterAppearance::conditions())
        )
}
pub fn is_dragonstone(equipped: engage::app::ItemData) -> bool {
    if !equipped.is_null() { equipped.is_dragon() || (equipped.get_iid().to_rust_string().contains("チキ") && equipped.get_kind().value == 9) }
    else { false }
}