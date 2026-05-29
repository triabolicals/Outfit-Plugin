use engage::{combat::CharacterAppearance, ut::Ut};
use outfit_core::anim::AnimData;
use transform::has_enemy_tiki;
use crate::assets::transform::is_dragonstone;
use super::*;

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
#[unity::hook("Combat", "CharacterAppearance", "ModifyColors")]
pub fn modify_colors(this: &mut CharacterAppearance, go: &GameObject, _: OptionalMethod) {
    call_original!(this, go, None);
    let data = UnitAssetMenuData::get();
    let mut rgb: Option<[u8; 3]> = None;
    for x in 0..6 {
        rgb = None;
        let j = 8 + x;
        let i = 4*j;
        if data.is_preview {
            if data.preview.color_preview[i+3] == 1 {
                rgb = Some([data.preview.color_preview[i], data.preview.color_preview[i+1], data.preview.color_preview[i+2]]);
            }
            else if data.preview.preview_data.colors[j].values[3] != 0 {
                rgb = Some([data.preview.preview_data.colors[j].values[0], data.preview.preview_data.colors[j].values[1], data.preview.preview_data.colors[j].values[2]]);
            }
        }
        else if let Some(data) = UnitAssetMenuData::get_by_person_data(this.person_hash, false) {
            if let Some(profile) = data.profile.get(data.profile_index(false) as usize) {
                if profile.colors[j].values[3] != 0 {
                    rgb = Some([profile.colors[j].values[0], profile.colors[j].values[1], profile.colors[j].values[2]]);
                }
            }
        }
        if let Some(rgb) = rgb {
            let r = rgb[0] as f32 / 255.0;
            let g = rgb[1] as f32 / 255.0;
            let b = rgb[2] as f32 / 255.0;
            if let Some(m) = get_mt_eye(go) { m.set_color(EYE_COLORS[x], Color::new(r, g, b, 1.0)); }
        }
    }
}
fn get_mt_eye(go: &GameObject) -> Option<&'static &'static Material2> {
    go.get_component_in_children::<SkinnedMeshRenderer>(true).iter()
        .flat_map(|smr| Ut::get_instance_materials2(smr).iter())
        .find(|v| v.get_name().to_string().starts_with("MtEye"))
}
const EYE_COLORS: [&str; 6] = ["_BaseColor", "_BlackColor", "_DecalColor1", "_DecalColor2", "_DecalColor3", "_DecalColor4"];