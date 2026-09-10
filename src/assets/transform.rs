use super::*;
use engage::{
    combat::{
        characterappearance::*, combatrecord::*, charactergamestatus::*,
        CombatStyle, Side, WeaponStyle,
    },
    app::{
        AssetTable_Modes, BattleCalculator, BattleSide_Type, ItemData, Unit, UnitItem,
        IGodDataMethods, IGodUnit, IItemDataMethods, IJobDataMethods, IPersonDataMethods, IUnitItem, IBitField32,
    },
    system::collections::generic::IList_1Methods,
};
use outfit_core::anim::SpecialAttackType;

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
pub fn has_enemy_tiki(unit: Unit) -> bool {
    let god_unit = unit.get_god_unit();
    if !god_unit.is_null() { god_unit.m_data().get_gid().to_rust_string().contains("敵チキ") }
    else { unit.get_pid().to_rust_string().contains("チキ") }
}

pub fn is_tiki_engage(unit: Unit) -> bool {
    unit.is_engaging_2() && {
        let god_unit = unit.get_god_unit();
        if !god_unit.is_null() { god_unit.m_data().get_gid().to_rust_string().contains("チキ") } else { false }
    }
}

fn set_transform_appearance(this: CombatRecord, unit: Unit, calculator: BattleCalculator, side: i32, calc_side: BattleSide_Type, distance: i32) {
    let array = Array::<Il2CppString>::from_slice(&[Il2CppString::new("コンバット"), Il2CppString::new("竜石")]).unwrap();
    let drag = this.get_game_status_dragonize().get(side as usize);
    drag.import_4(side, calculator, calc_side, distance);
    let result = AssetTable_Result::get_from_unit(AssetTable_Modes::combat(), unit, array);
    let female = get_outfit_data().get_dress_gender(result.get_dress_model()).value == 2;
    anim::AnimData::remove(result, true, true);
    let transform_anim = anim::AnimData::get_transforming_anim(unit.is_engaging_2(), female);;
    result.set_body_anim(transform_anim);
    result.get_body_anims().add(transform_anim.into());
    drag.set_appearance(CharacterAppearance::create_from_result(result, distance));
}
fn bullet_attack_result(unit: Unit, item: ItemData) -> AssetTable_Result {
    let array = Array::<Il2CppString>::from_slice(&[Il2CppString::new("コンバット"), Il2CppString::new("弾丸")]).unwrap();
    let result = AssetTable_Result::get_from_unit(AssetTable_Modes::combat(), unit, array);
    if item.is_bullet() { AssetTable::s_condition_flags().add_3(item); }
    result.commit(AssetTable_Modes::combat());
    result.replace(AssetTable_Modes::combat());
    anim::AnimData::remove(result, true, true);
    let body_anims = result.get_body_anims();
    body_anims.clear();
    let female = get_outfit_data().get_dress_gender(result.get_dress_model()).value == 2;
    result.get_body_anims().add(anim::AnimData::bullet_anim(false, female).into());
    if unit.is_engaging_2() {
        result.get_body_anims().add(anim::AnimData::bullet_anim(true, female).into());
    }
    result
}

#[skyline::hook(offset=0x029285f0)]
pub fn change_dragon2(this: CombatRecord, calc_side: BattleSide_Type, param_3: &CombatRecordDisplayClass85, method_info: OptionalMethod) {
    call_original!(this, calc_side, param_3, method_info);
    let calculator = this.get_calculator();
    let side = Side::convert_from(calc_side, this.get_is_enemy_attack());
    let distance = if this.get_map_distance() <= 1 { 1 }  else { 2 };
    let game_status = this.get_game_status();
    let gs = game_status.get(side as usize);
    if gs.is_null() { return; }
    let unit = gs.get_unit();
    if unit.is_null() { return; }
    let person = unit.get_person();
    let mut style = this.get_combat_style().value;
    if !gs.appearance().is_null(){
        if !gs.appearance().assets().is_null() { if gs.appearance().get_race().value != 1 { return; } }
    }
    if person.get_pid().to_rust_string().contains("チキ") { return; }
    let state = AssetTable_ConditionFlags::get_state(unit);
    let god_unit = unit.get_god_unit();
    match state.value {
        2 => {
            if god_unit.is_null() { return; }
        }
        0|1 => {
            let item = gs.get_weapon();
            if item.is_null() { return; }
            let item = item.m_item();
            match SpecialAttackType::determine_from_unit(unit, item) {
                SpecialAttackType::Bullet => {
                    style &= !(1 << 22);
                    let drag = this.get_game_status_dragonize().get(side as usize);
                    if !drag.is_null() { drag.clear(); }
                    gs.set_appearance(CharacterAppearance::create_from_result(bullet_attack_result(unit, item), distance));
                    gs.appearance().set_weapon_style(WeaponStyle::magic());
                }
                SpecialAttackType::Transform => {
                    if is_tiki_dragon_weapon(item) || unit.get_job().get_jid().contains("チキ"){
                        style |= 1 << 22;
                        set_transform_appearance(this, unit, calculator, side, calc_side, distance);
                        let result = AssetTable_Result::get_for_talk_2(unit);
                        let colors = [result.get_mask_color100(), result.get_mask_color075(), result.get_mask_color050(), result.get_mask_color025()];
                        let sound = il2str(result.get_sound().voice_id);
                        result.clear();
                        let conditions = AssetTable::s_condition_flags();
                        if !god_unit.is_null() {
                            let data = god_unit.m_data();
                            let gid = data.get_gid().to_rust_string();
                            if data.get_force_type().value == 1 && (gid.contains("M0") || gid.contains("E00")) {
                                conditions.add_2("PID_E001_Boss_竜化");
                            }
                            else { conditions.add_2("AID_Person_チキ_竜化"); }
                        }
                        else { conditions.add_2("AID_Person_チキ_竜化"); }
                        conditions.add_3(item);
                        result.commit(AssetTable_Modes::combat());
                        result.replace(AssetTable_Modes::combat());
                        if let Some(sounds) = sound {
                            let mut s = result.get_sound();
                            s.voice_id = sounds.into();
                            result.set_sound(s);
                        }
                        if !unit.is_engaging_2() && unit.get_force_type().value == 0 {
                            result.set_mask_color100(colors[0]);
                            result.set_mask_color075(colors[1]);
                            result.set_mask_color050(colors[2]);
                            result.set_mask_color025(colors[3]);
                        }
                        gs.set_appearance(CharacterAppearance::create_from_result(result, distance));
                    }
                    else if let Some(transform) = get_transformation2(unit, gs.get_weapon()){
                        style |= 1 << 22;
                        set_transform_appearance(this, unit, calculator, side, calc_side, distance);
                        gs.set_appearance(CharacterAppearance::create_from_result(transform, distance));
                    }
                }
                _ => {}
            }
        }
        _ => {}
    }
    this.set_combat_style(CombatStyle{value: style});
}

#[skyline::hook(offset=0x02928bc0)]
pub fn transformation_chain_atk(this: CombatRecord, calc_side: i32, param_3: &CombatRecordDisplayClass87, method_info: OptionalMethod) {
    let chain_atk_index = this.get_chain_attack_count();
    call_original!(this, calc_side, param_3, method_info);
    let count = this.get_chain_attack_count();
    if chain_atk_index < count {
        let side = this.get_game_status_chain_atk().get(chain_atk_index as usize);
        if !side.is_null() {
            let unit = side.get_unit();
            if !unit.is_null() {
                if let Some(appearance) = get_transformation2(side.get_unit(), side.get_weapon()) {
                    side.set_appearance(CharacterAppearance::create_from_result(appearance, 1));
                }
            }
        }
    }
}
pub fn get_transformation2(unit: Unit, weapon: UnitItem) -> Option<AssetTable_Result> {
    let item = weapon.m_item();
    let kind = SpecialAttackType::determine_from_unit(unit, weapon.m_item());
    match kind {
        SpecialAttackType::Transform => {
            get_outfit_data().get_combat_transformation(unit, item)
                .or_else(||
                    Some(
                        AssetTable_Result::get_from_pid(
                            AssetTable_Modes::combat(),if unit.get_gender().value == 2 { "PID_エル_竜化" } else { "PID_ラファール_竜化"},
                            CharacterAppearance::conditions()
                        )
                    )
                )
        }
        SpecialAttackType::Bullet => { if item.is_bullet() { None } else { Some(bullet_attack_result(unit, item)) } }
        SpecialAttackType::NormalAttack => { None }
    }
}
pub fn is_tiki_dragonstone(item_data: ItemData) -> bool {
    if item_data.is_null() { false }
    else { item_data.get_kind().value == 9 && item_data.get_iid().to_rust_string().contains("チキ") }
}
pub fn is_dragonstone(equipped: ItemData) -> bool {
    if equipped.is_null() { false } else {
        equipped.is_dragon() || is_tiki_dragonstone(equipped) || (equipped.get_kind().value == 9 && !equipped.is_bullet() && equipped.get_flag().m_value() & 0x40000 == 0)
    }
}
fn is_tiki_dragon_weapon(item: ItemData) -> bool {
    if item.is_null() { false } else { item.get_iid().to_rust_string().contains("チキ") && item.get_kind().value == 9 }
}