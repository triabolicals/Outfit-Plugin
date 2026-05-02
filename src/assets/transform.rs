use super::*;
use engage::{combat::{CharacterAppearance, CombatSide, CombatRecord}, battle::{BattleCalculator, BattleSideType}};

pub const MONSTERS: [&str; 8] = ["JID_幻影飛竜", "JID_異形飛竜", "JID_幻影狼", "JID_異形狼",  "JID_E006ラスボス", "JID_幻影竜", "JID_異形竜", "JID_邪竜"];
pub const MONSTER_PERSONS: [&str; 8] = [
    "PID_G000_幻影飛竜", "PID_E004_異形兵_異形飛竜", "PID_G000_幻影狼", "PID_E001_異形兵_異形狼",
    "PID_E006_Boss", "PID_S006_幻影竜", "PID_M019_異形竜", "PID_M026_ソンブル_竜型"
];
pub const SCALE: [f32; 8] = [  1.0, 1.0, 1.0, 1.0, 0.40, 1.0, 1.0, 0.40];


pub struct CombatRecordDisplayClass85 {
    pub this: &'static CombatRecord,
    pub calc: &'static BattleCalculator,
}

#[repr(C)]
pub struct CombatRecordDisplayClass87 {
    pub calc: &'static BattleCalculator,
    pub pre_index: i32,
    pub this: &'static CombatRecord,

}
pub fn is_monster_class(unit: &Unit) -> bool {
    let jid = unit.get_job().jid.to_string();
    if unit.person.bmap_size > 1 || unit.person.gender == 0 { false }
    else { MONSTERS.iter().any(|&monster| monster == jid)  }
}
pub fn has_enemy_tiki(unit: &Unit) -> bool {
    if let Some(god_unit) = unit.god_unit { god_unit.data.gid.to_string().contains("敵チキ") }
    else if let Some(god_unit) = unit.god_link { god_unit.data.gid.to_string().contains("敵チキ") }
    else { unit.person.pid.to_string().contains("チキ") }
}

pub fn is_tiki_engage(unit: &Unit) -> bool {
    unit.status.value & 8388608 != 0 && unit.god_unit.is_some_and(|g_unit| g_unit.data.gid.contains("チキ"))
}

#[skyline::hook(offset=0x029285f0)]
pub fn change_dragon2(this: &mut CombatRecord, calc_side: BattleSideType, param_3: &CombatRecordDisplayClass85, method_info: OptionalMethod) {
    call_original!(this, calc_side, param_3, method_info);
    let side = CombatSide::convert_from(calc_side, this.is_enemy_attack != 0);
    if side < 0 { return; }
    let side = side as usize;
    if this.game_status[side].unit.is_some_and(|u|
        u.person.gender == 0 || u.person.bmap_size > 1 ||
        u.person.pid.str_contains("PID_E001_Boss") ||
        u.person.pid.str_contains("PID_E006_Hide8") ||
        u.person.pid.str_contains("チキ_竜化") ||
        u.person.pid.str_contains("チキ")
    ){
        return;
    }
    let distance = if this.map_distance < 1 { 1 } else if  this.map_distance > 2 { 2 } else { this.map_distance };
    if let Some(unit) = this.game_status[side].get_unit() {
        if let Some(g_unit) = unit.god_link.or(unit.god_unit) {
            let status = AssetTableConditionFlags::get_state(unit);
            if g_unit.data.gid.contains("チキ") && status >= AssetTableStates::Engaging { return; }
        }
        let can_dragon_stone = unit.job.mask_skills.find_sid("SID_竜石装備").is_some() && unit.job.get_max_weapon_level(9) > 1;
        if this.game_status[side].weapon.is_some_and(|i| is_dragonstone(Some(i.item)) || (i.item.kind == 9 &&  can_dragon_stone)){
            this.combat_style |= 1 << 22;
            this.dragonize[side].import(side as i32, param_3.calc, calc_side, distance);
            this.game_status[side].appearance = CharacterAppearance::create_from_result(get_transform_result(unit), distance);
            return;
        }
        let db = get_outfit_data();
        if let Some(monster_data) = db.dress.transform.iter().find(|x| !x.is_transform && unit.job.parent.hash == x.hash){
            this.combat_style |= 1 << 22;
            this.dragonize[side].import(side as i32, param_3.calc, calc_side, distance);
            this.game_status[side].appearance = CharacterAppearance::create_from_result(monster_data.get_result(2, unit), distance);
        }
    }
}

#[skyline::hook(offset=0x02928bc0)]
pub fn transformation_chain_atk(this: &mut CombatRecord, calc_side: i32, param_3: &CombatRecordDisplayClass87, method_info: OptionalMethod) {
    let chain_atk_index = this.chain_attack_count as usize;
    call_original!(this, calc_side, param_3, method_info);
    let count = this.chain_attack_count as usize;
    if chain_atk_index < count && chain_atk_index < this.chain_atk.len()  {
        let side = this.chain_atk[chain_atk_index].side as usize;
        if let Some(unit) = this.chain_atk[chain_atk_index].get_unit() {
            let can_dragon_stone = unit.job.mask_skills.find_sid("SID_竜石装備").is_some() && unit.job.get_max_weapon_level(9) > 1;
            if this.chain_atk[side].weapon.is_some_and(|i| is_dragonstone(Some(i.item)) || (i.item.kind == 9 &&  can_dragon_stone)){
                this.chain_atk[side].appearance = CharacterAppearance::create_from_result(get_transform_result(unit), 1);
                return;
            }
            let db = get_outfit_data();
            if let Some(monster_data) = db.dress.transform.iter().find(|x| !x.is_transform && unit.job.parent.hash == x.hash){
                this.chain_atk[side].appearance = CharacterAppearance::create_from_result(monster_data.get_result(2, unit), 1);
            }
        }
    }
}
fn get_transform_result(unit: &Unit) -> &'static mut AssetTableResult {
    let db = get_outfit_data();
    let job = unit.job.parent.hash;
    db.dress.transform.iter().find(|x| x.is_transform && job == x.hash).map(|data| data.get_result(2, unit))
        .unwrap_or(
            AssetTableResult::get_from_pid(2,if unit.person.get_gender() == 2 { "PID_エル_竜化" } else { "PID_ラファール_竜化"}, CharacterAppearance::get_constions(None))
        )
}
pub fn is_dragonstone(equipped: Option<&ItemData>) -> bool {
    equipped.is_some_and(|i| i.flag.value & 0x4000000 != 0 || (i.iid.str_contains("チキ") && i.kind == 9 ))
}