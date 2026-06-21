use engage_il2cpp::app::{AssetTable_Modes, IAssetTable_ResultMethods, IBitField32, IGodDataMethods, IGodUnit, IJobDataMethods, IPersonDataMethods, IStructData_1Methods, IUnitMethods, Unit_Status};
use engage_il2cpp::combat::{ICharacterAssetForm, ICharacterMethods};
use unity2::{Cast, IlNull};
use outfit_core::anim::AnimData;
use super::*;

pub fn commit_for_unit_dress(
    result: AssetTable_Result,
    mode: i32,
    unit: engage_il2cpp::app::Unit,
    equipped: engage_il2cpp::app::ItemData,
    conds: unity2::Array<unity2::Il2CppString>,
    conditions: &mut AssetConditions
) {
    let mmode = AssetTable_Modes{value: mode};
    let person = unit.get_person();
    let flags = person.get_flag().m_value();
    if conditions.flags.contains(AssetFlags::Monster) || flags & 64 != 0 || mode == 3 || person.get_gender().value & 3 == 0 || person.get_bmap_size() > 1 {
        result.commit_2(mmode, unit.get_person(), unit.get_job(), equipped);
        return;
    }
    let condition_unit =
        if conditions.flags.contains(AssetFlags::Vision) {
            let owner = engage_il2cpp::app::UnitUtil::get_vision_owner(unit);
            if !owner.is_null() { owner }
            else { unit }
        } else { unit };

    if conditions.flags.contains(AssetFlags::MapTransform) && mode == 1 {
        let jid = condition_unit.get_job().get_jid().to_rust_string();
        /*
        if has_enemy_tiki(unit) {
            result.setup_4(1, engage_il2cpp::app::PersonData::get("PID_E001_Boss".into()), conds);
            result.get_sound().voice_id = unity2::Il2CppString::null();
            return
        }
        else if is_dragonstone(equipped) && equipped.is_some_and(|i| i.iid.str_contains("チキ") && i.kind == 9) {
            result.setup_for_person(1, PersonData::get("PID_G001_チキ_竜化"),conds);
        }
        else if !get_outfit_data().apply_monster_asset(result, unit, mode) {
            if jid == "JID_裏邪竜ノ子" || unit.get_dress_gender() == engage_il2cpp::app::Gender::male() {
                result.setup_for_person_job_item(1, PersonData::get("PID_ラファール_竜化"), Some(condition_unit.job), None, conds);
            }
            else { result.setup_for_person_job_item(1, PersonData::get("PID_エル_竜化"), Some(condition_unit.job), None, conds); }
        }

         */
        return
    }
    let engaged = condition_unit.is_engaging_2();
    if engaged && conditions.flags.contains(AssetFlags::EngageTiki) {
        AssetFlags::remove_unit_accessories(condition_unit);
        result.commit_2(mmode, condition_unit.get_person(), condition_unit.get_job(), equipped);
        return;
    }
    let mut profile_flag = 0;
    let db = get_outfit_data();
    if let Some(data) = UnitAssetMenuData::get_unit_data(condition_unit) {
        let god_unit = condition_unit.get_god_unit();
        if !god_unit.is_null() && engaged {
            let profile_flag = data.get_active_flag(engaged);
            if profile_flag & 256 != 0 { conditions.flags.set_condition_flag(AssetFlags::Engaged, false); }
            if profile_flag & 6 == 2 { conditions.remove_god_eid_conditions(); }
            else if profile_flag & 6 == 4 {
                let data = god_unit.m_data();
                let gid = data .get_gid().to_string();
                conditions.remove_god_eid_conditions();
                AssetFlags::set_person_conditions(condition_unit.get_person(), false);
                AssetFlags::set_condition_key(gid, true);
                AssetFlags::set_condition_key(data.get_mid(), true);
                AssetFlags::set_condition_key(data.get_asset_id(), true);
                conditions.flags.set_condition_flag(AssetFlags::Engaged, false);
                let gender = if data.get_female() == 1 { engage_il2cpp::app::Gender::female() } else { engage_il2cpp::app::Gender::male() };
                conditions.flags.set_gender(gender);
                result.commit_2(mmode, condition_unit.get_person(), engage_il2cpp::app::JobData::null(), equipped);
                db.correct_anims(result, unit, profile_flag, conditions);
                return;
            }
        }
        if UnitAssetMenuData::get().is_preview {
            if UnitAssetMenuData::get().is_shop_combat { AssetFlags::remove_unit_accessories(condition_unit); }
            result.commit_2(mmode, condition_unit.get_person(), condition_unit.get_job(), equipped);
        }
        else {
            result.commit_2(mmode, condition_unit.get_person(), condition_unit.get_job(), equipped);
        }
        profile_flag = data.get_active_flag(conditions.flags.contains(AssetFlags::Engaged));
        // println!("Setting AssetSets");
        UnitAssetMenuData::set_assets(result, condition_unit, conditions);
        // println!("Assets Set");
    }
    else {
        result.commit_2(mmode, condition_unit.get_person(), condition_unit.get_job(), equipped);
        db.adjust_dress(result, condition_unit, conditions);
    }
    if is_monster_body(result) {
        if conditions.flags.contains(AssetFlags::Vision) {
            result.setup_5(mmode, engage_il2cpp::app::PersonData::get("PID_S004_リン".into()), engage_il2cpp::app::JobData::get("JID_紋章士_リン".into()), equipped, conds);
            db.anims.set_vision_anims(result, engage_il2cpp::app::Gender::female(), mode);
        }
        return;
    }
    else { hair_adjustment(result); }
    if condition_unit.check_status(Unit_Status::engage_attack()) && conditions.mode == 2{
        AnimData::adjust_engage_atk(result, db.get_dress_gender(result.get_dress_model()));
        return;
    }
    if conditions.flags.contains(AssetFlags::CombatTranforming) { AnimData::remove(result, true, true); }
    // println!("Correction Anims");
    db.correct_anims(result, unit, profile_flag, conditions);
    // println!("Finished Correction Anims");
}
fn hair_adjustment(result: AssetTable_Result) {
    /*
    if !result.hair_model.is_null() {
        if !result.hair_model.contains("null") {
            if result.accessory_list.list.iter()
                .any(|acc| acc.model.is_some_and(|model| model.to_string().contains("Hair"))) {
                result.hair_model = "uHair_null".into();
            }
        }
    }

     */
}
#[unity::hook("Combat", "CharacterAppearance", "ModifyColors")]
pub fn modify_colors(this: engage_il2cpp::combat::CharacterAppearance, go: engage_il2cpp::unity_engine::GameObject, _: unity2::OptionalMethod) {
    get_head_hair_colors(go);
    call_original!(this, go, None);
    apply_preview_head_hair_color(this, go);
}
#[skyline::hook(offset=0x2b011f0)]
fn combat_character_play_facial(this: engage_il2cpp::combat::Character, state_hash: i32, transition: f32, optional_method: unity2::OptionalMethod) {
    if !UnitAssetMenuData::get().is_preview {
        let builder = this.get_builder().appearance();
        let hash = unity2::field_get_value_at_offset::<i32>(builder, 0xd4);
        if let Some(person) = UnitAssetMenuData::get_current_profile(hash) {
            if let Some(pos) = FACIAL_STATES.iter().enumerate().position(|(i, s)| s.1 == state_hash && i < 4){
                let exp = person.expression[pos] as usize;
                if exp > 0 && exp != (pos + 1) {
                    call_original!(this, FACIAL_STATES[exp -1 ].1, transition, optional_method);
                    return;
                }
            }
        }
    }
    call_original!(this, state_hash, transition, optional_method);
}

