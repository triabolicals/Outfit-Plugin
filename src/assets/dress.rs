use outfit_core::anim::AnimData;
use transform::has_enemy_tiki;
use crate::assets::transform::is_dragonstone;
use super::*;

/*
fn is_preview_unit(unit: &Unit) -> bool {
   unit.force.is_some_and(|x| (1 << x.force_type) & 25 != 0) && unit.status.value & 35184372088832 == 0
}
 */
pub fn commit_for_unit_dress(
    result: &mut AssetTableResult,
    mode: i32,
    unit: &mut Unit,
    equipped: Option<&ItemData>,
    conds: &Array<&Il2CppString>,
    conditions: &mut AssetConditions
) {
    if conditions.flags.contains(AssetFlags::Monster) || unit.person.flag.value & 64 != 0 || mode == 3 || unit.person.gender & 3 == 0 || unit.person.bmap_size > 1 {
        result.commit(mode, Some(unit.person), Some(unit.job), equipped);
        return;
    }
    let condition_unit = if conditions.flags.contains(AssetFlags::Vision) { UnitUtil::get_vision_owner(unit).unwrap_or(unit) } else { &unit };
    if conditions.flags.contains(AssetFlags::MapTransform) && mode == 1 {
        let jid = condition_unit.job.jid.to_string();
        if has_enemy_tiki(unit) {
            result.setup_for_person(1, PersonData::get("PID_E001_Boss"), conds);
            result.sound.voice = None;
            return
        }
        else if is_dragonstone(equipped) && equipped.is_some_and(|i| i.iid.str_contains("チキ") && i.kind == 9) {
            result.setup_for_person(1, PersonData::get("PID_G001_チキ_竜化"),conds);
        }
        else if !get_outfit_data().apply_monster_asset(result, unit, mode) {
            if jid == "JID_裏邪竜ノ子" || unit.get_dress_gender() == Gender::Male {
                result.setup_for_person_job_item(1, PersonData::get("PID_ラファール_竜化"), Some(condition_unit.job), None, conds);
            }
            else { result.setup_for_person_job_item(1, PersonData::get("PID_エル_竜化"), Some(condition_unit.job), None, conds); }
        }
        return
    }
    let engaged = condition_unit.status.value & 8388608 != 0;
    if engaged && conditions.flags.contains(AssetFlags::EngageTiki) {
        AssetFlags::remove_unit_accessories(condition_unit);
        result.commit(mode, Some(condition_unit.person), Some(condition_unit.job), equipped);
        return;
    }
    let mut profile_flag = 0;
    let db = get_outfit_data();
    if let Some(data) = UnitAssetMenuData::get_unit_data(condition_unit) {
        if let Some(god) = condition_unit.god_link.or(condition_unit.god_unit).filter(|_| engaged  ) {
            let profile_flag = data.get_active_flag(engaged);
            if profile_flag & 256 != 0 { conditions.flags.set_condition_flag(AssetFlags::Engaged, false); }
            if profile_flag & 6 == 2 { conditions.remove_god_eid_conditions(); }
            else if profile_flag & 6 == 4 {
                let gid = god.data.gid.to_string();
                conditions.remove_god_eid_conditions();
                AssetFlags::set_person_conditions(condition_unit.person, false);
                AssetFlags::set_condition_key(gid, true);
                AssetFlags::set_condition_key(god.data.mid, true);
                AssetFlags::set_condition_key(god.data.asset_id, true);
                conditions.flags.set_condition_flag(AssetFlags::Engaged, false);
                let gender = if god.data.female == 1 { Gender::Female } else { Gender::Male };
                conditions.flags.set_gender(gender);
                result.commit(mode, Some(condition_unit.person), None, equipped);
                db.correct_anims(result, unit, profile_flag, conditions);
                return;
            }
        }
        if UnitAssetMenuData::get().is_preview {
            if UnitAssetMenuData::get().is_shop_combat { AssetFlags::remove_unit_accessories(condition_unit); }
            result.commit(mode, Some(condition_unit.person), Some(condition_unit.job), equipped);
        }
        else {
            result.commit(mode, Some(condition_unit.person), Some(condition_unit.job), equipped);
            db.accessory_conditions.commit_accessories(result, condition_unit, mode);
        }
        profile_flag = data.get_active_flag(conditions.flags.contains(AssetFlags::Engaged));
        UnitAssetMenuData::set_assets(result, condition_unit, conditions);
    }
    else {
        result.commit(mode, Some(condition_unit.person), Some(condition_unit.job), equipped);
        db.adjust_dress(result, &condition_unit, conditions);
    }
    if is_monster_body(result) {
        if conditions.flags.contains(AssetFlags::Vision) {
            result.setup_for_person_job_item(mode, PersonData::get("PID_S004_リン"), JobData::get("JID_紋章士_リン"), equipped, conds);
            db.anims.set_vision_anims(result, Gender::Female, mode);
        }
        return;
    }
    else { hair_adjustment(result); }
    if condition_unit.status.value & UnitStatusField::EngageAttack  != 0 && conditions.mode == 2{
        AnimData::adjust_engage_atk(result, db.get_dress_gender(result.dress_model));
        return;
    }
    if conditions.flags.contains(AssetFlags::CombatTranforming) { AnimData::remove(result, true, true); }
    db.correct_anims(result, unit, profile_flag, conditions);
}
fn hair_adjustment(result: &mut AssetTableResult) {
    if !result.hair_model.is_null() {
        if !result.hair_model.contains("null") {
            if result.accessory_list.list.iter()
                .any(|acc| acc.model.is_some_and(|model| model.to_string().contains("Hair"))) {
                result.hair_model = "uHair_null".into();
            }
        }
    }
}
/*
fn eve_sforgia_correction(result: &mut AssetTableResult, mode: i32) {
    if mode != 1 { return; }
    if !result.body_model.is_null() {
        let body = result.body_model.to_string();
        if body.contains("c451") { result.body_model = body.replace("c451", "c000").into(); }
        if body.contains("c151") { result.body_model = body.replace("c151", "c000").into(); }
    }
}

 */
