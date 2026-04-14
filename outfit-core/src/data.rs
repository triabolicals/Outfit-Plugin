pub use engage::{
    unit::Unit,
    gamedata::{accessory::AccessoryData, assettable::AssetTable, Gamedata, GodData, JobData, PersonData},
    mess::Mess,
    resourcemanager::*,
};
// use std::fs::File;

// use assets::*;
pub use super::*;

mod color;
mod hashes;
mod item;
pub mod anim;
pub(crate) mod dress;
pub mod unit_acc;
mod util;
mod list;

pub use color::*;
pub use hashes::*;
pub use item::*;
pub use list::*;
pub use unit_acc::AccessoryConditions;

use engage::gamedata::assettable::{AssetTableResult, AssetTableStaticFields};
use engage::gamevariable::GameVariableManager;
use engage::random::Random;
use anim::AnimData;
use crate::data::dress::{DressData, JobDressData};
use crate::enums::Mount;

pub const KINDS: [&str; 8] = ["uBody_", "uHead_", "uHair_", "uAcc_spine2_Hair", "uAcc_head_", "uAcc_spine", "uAcc_Eff", "uAcc_shield_"];
pub const NULL: [&str; 4] = ["uBody_null", "uHead_null", "uHair_null", "uAcc_head_null"];

const AOC: [&str; 4] = ["Info", "Talk", "Demo", "Hub"];

pub struct OutfitData {
    pub hashes: OutfitHashes,
    pub accessory_conditions: AccessoryConditions,
    pub dress: DressData,
    pub item: Vec<ItemAsset>,
    pub anims: AnimData,
    pub list: OutfitLists,
    pub labels: AssetLabelTable,
}

impl OutfitData {
    pub fn init() -> Self {
        let new_labels = AssetLabelTable::new();
        let mut new_list = OutfitLists::new();
        let mut hashes = OutfitHashes::new();
        AssetTable::get_list().unwrap().iter()
            .filter_map(|x| x.voice)
            .for_each(|v| {
                let hash = v.get_hash_code();
                if !hashes.voice.contains_key(&hash) { hashes.voice.insert(hash, v.to_string()); }
            });

        let mut assets: Vec<_> = ResourceManager::class().get_static_fields::<ResourceManagerStaticFields>()
            .files
            .entries.iter()
            .filter(|x| x.key.is_some_and(|x| !x.str_contains("null") && (x.str_contains("UAS_") || x.str_contains("Item/Acc/") || x.str_contains("Unit/Model/") || x.str_contains("AOC_") || x.str_contains("uRig"))))
            .map(|x| { x.key.unwrap().to_string() })
            .flat_map(|x| x.split("/").last().map(|v| v.to_string()))
            .collect();
        let rig_ends = ["M1", "M", "F", "F1"];
        // Collect all rigs, oHairs and oBody
        hashes.rigs = assets.extract_if(.., |s| !s.contains("Wolf") && !s.contains("Drag") && s.contains("uRig_") && (s.contains("Humn") || rig_ends.iter().any(|x| s.ends_with(*x))))
            .map(|s| (Il2CppString::new(s.as_str()).get_hash_code(), s)).collect();

        hashes.o_hair = assets.extract_if(.., |s| s.contains("oHair_h") || s.contains("oHair_dummy")).map(|s| (Il2CppString::new(s.as_str()).get_hash_code(), s)).collect();
        hashes.o_body = assets.extract_if(.., |s| s.contains("oBody_")).map(|s| (Il2CppString::new(s.as_str()).get_hash_code(), s)).collect();
        hashes.o_acc = assets.extract_if(.., |s| s.contains("oAcc_")).map(|s| (Il2CppString::new(s.as_str()).get_hash_code(), s)).collect();

        EMBLEM.iter().for_each(|(i, d)| {
            let mpid = format!("MPID_{}", i);
            if let Some(s) = get_remove(&mut assets, format!("uBody_{}1AM", d).as_str()) {
                new_list.add_engaged_body(mpid.as_str(), s.as_str(), false);
                hashes.add_body(s.as_str(), false);
            }
            if let Some(s) = get_remove(&mut assets, format!("uBody_{}1AF", d).as_str()) {
                new_list.add_engaged_body(mpid.as_str(), s.as_str(), true);
                hashes.add_body(s.as_str(), true);
            }
        });

        [("File", "Filene"), ("Brod", "Brodia"), ("Irci", "Ircion"), ("Solu", "Solum"), ("Lith", "Lithos"), ("Swim", "Swimwear")]
            .iter().enumerate()
            .for_each(|(n, x)| {
                ["M", "F"].iter().enumerate().map(|(i, g)| (i == 1, g)).for_each(|(female, gen)| {
                    if n < 5 {
                        for y in 0..6 {
                            let i = y + 1;
                            let label_i = (y % 3) + 1;
                            let label = format!("MAID_{}{}{}{}", x.1, if y < 3 { "Formal" } else { "Casual" }, label_i, gen);
                            if let Some(asset) = get_remove(&mut assets, format!("uBody_{}{}{}_c000", x.0, i, gen).as_str()) {
                                hashes.add_body(asset.as_str(), female);
                                new_list.add_other_body(label.as_str(), asset.as_str(), female, 0, true);
                            }
                            if let Some(helm) = get_remove(&mut assets, format!("Helm{}{}{}", x.0, i, gen).as_str()) {
                                hashes.add_acc(helm.as_str(), None);
                                new_list.add(helm, false, Some(label.as_str()), 0);
                            }
                        }
                    } else {
                        for y in 1..4 {
                            while let Some(asset) = get_remove(&mut assets, format!("uBody_{}{}{}_c000", x.0, gen, y).as_str()) {
                                let label = format!("MAID_{}{}{}", x.1, y, gen);
                                hashes.add_body(asset.as_str(), female);
                                new_list.add_other_body(label.as_str(), asset.as_str(), female, 0, true);
                            }
                        }
                    }
                });
            });
        let mut section = 0;
        let mut voices = hashes.voice.iter().map(|v| v.1.clone()).collect::<Vec<String>>();
        let mut no_job_body = vec![];
        let mut no_job_body_f = vec![];
        include_str!("../data/labels2.txt").lines()
            .for_each(|line| {
                if line.starts_with("END") { section += 1; } else {
                    match section {
                        0 => {
                            if let Some((g, has_body)) = AssetGroup::new_job_group(line, &mut assets, &mut hashes, false) {
                                if has_body { new_list.job_m.push(g); } else { no_job_body.push(g); }
                            }
                            if let Some((g, has_body)) = AssetGroup::new_job_group(line, &mut assets, &mut hashes, true) {
                                if has_body { new_list.job_f.push(g); } else { no_job_body_f.push(g); }
                            }
                        },
                        1 => if let Some(g) = AssetGroup::new_character_group(line, &mut assets, &mut voices, &mut hashes) { new_list.char_m.push(g); },
                        2 => if let Some(g) = AssetGroup::new_character_group(line, &mut assets, &mut voices, &mut hashes) { new_list.char_f.push(g); },
                        4 => if let Some(g) = AssetGroup::new_aid_group(line, &mut assets, &mut hashes) { new_list.aids.push(g); },
                        5 => {
                            let mut spilt = line.split_whitespace();
                            if let Some((label, value)) = spilt.next().zip(spilt.next()) {
                                if let Some(asset) = get_remove(&mut assets, value) { new_list.add(asset, false, Some(label), 0); }
                            }
                        }
                        6 => { // manually uHead to oHair conversion
                            let mut spilt = line.split_whitespace();
                            if let Some(s) = spilt.next() {
                                let o_hash = hash_string(format!("oHair_{}", s));
                                while let Some(l) = spilt.next() {
                                    let u_asset =
                                        if l.starts_with("Hair") { format!("uAcc_spine2_{}", l) } else if l.starts_with("c") { format!("uHead_{}", l) } else { format!("uHair_{}", l) };
                                    let u_hash = hash_string(&u_asset);
                                    hashes.head_hair.insert(u_hash, o_hash);
                                }
                            }
                        }
                        7 => {  // Skin
                            let mut spilt = line.split_whitespace();
                            if let Some((head, color)) = spilt.next().zip(spilt.next()) {
                                let mut c = AssetColor::new();
                                color.split(",").flat_map(|v| v.parse::<i32>().ok())
                                    .enumerate()
                                    .for_each(|(i, s)| { c.values[i] |= s as u8; });
                                c.values[3] = 255;
                                head.split(",").for_each(|head| {
                                    let hash = hash_string(format!("uHead_c{}", head));
                                    new_list.skin.insert(hash, c);
                                });
                            }
                        }
                        _ => {}
                    }
                }
            });
        // Adding classes that do not have ubodies
        new_list.job_count.0 = new_list.job_m.len() as i32;
        new_list.job_count.1 = new_list.job_f.len() as i32;
        no_job_body.into_iter().for_each(|g| { new_list.job_m.push(g); });
        no_job_body_f.into_iter().for_each(|g| { new_list.job_f.push(g); });

        voices.iter().for_each(|a| {
            new_list.other.push(
                OtherAssetItem {
                    label: a.to_string(),
                    asset: AssetItem { hash: hash_string(a), count: 0, flags: AssetItemFlags::empty(), kind: AssetType::Voice },
                    is_mess: true,
                    female: false,
                }
            )
        });
        // Add Char AOC to hashes
        new_list.char_m.iter().for_each(|g| {
            g.list.iter().filter(|x| x.kind.to_index() >= 30 && x.kind.to_index() < 34)
                .for_each(|a| { hashes.aoc_m.insert(a.hash); });
        });
        new_list.char_f.iter().for_each(|g| {
            g.list.iter().filter(|x| x.kind.to_index() >= 30 && x.kind.to_index() < 34)
                .for_each(|a| { hashes.aoc_f.insert(a.hash); });
        });
        include_str!("../data/heads.txt").lines()
            .enumerate()
            .map(|(i, x)| (i & 1 != 0, x.split_whitespace()))
            .for_each(|(female, suffix)| {
                suffix.for_each(|ss| {
                    AOC.iter().enumerate().for_each(|(_, a)| {
                        assets.extract_if(.., |s| s.contains(format!("AOC_{}", a).as_str()) && s.contains(ss) && !s.contains("Photo") && !s.contains("Refresh"))
                            .for_each(|s| {
                                hashes.add_aoc(s.as_str(), female);
                                new_list.add(s, female, None::<String>, 0);
                            });
                    });
                });
            });
        let mut npcs = (801..820).collect::<Vec<usize>>();
        npcs.extend(850..860);
        for x in npcs {
            for _ in 0..2 {
                if let Some(asset) = get_remove(&mut assets, format!("uHead_c{}", x).as_str()) {
                    hashes.add_head(asset.as_str());
                    new_list.add(asset.as_str(), false, None::<String>, 0);
                }
            }
        }
        for x in [858, 863, 864, 865, 870] {
            if let Some(asset) = get_remove(&mut assets, format!("_Hair{}", x).as_str()) {
                hashes.add_hair(asset.as_str(), None);
                new_list.add(asset.as_str(), false, None::<String>, 0);
            }
        }
        let kinds = ["ubody_", "uhead_c", "uhair_h", "uacc_spine2_hair", "uacc_head_", "uacc_spine", "uacc_eff", "uacc_shield_"];
        let female = ["f_c", "f1_c", "f2_c", "f3_c", "f4_c"];
        let male = ["m_c", "m1_c", "m2_c", "m3_c", "m4_c"];
        let mut remove = vec![];
        assets.iter().enumerate()
            .filter(|(_, s)|{
                let lower = s.to_lowercase();
                !s.contains("null") && !lower.contains("box") && !lower.contains("dummy") && kinds.iter().any(|k| lower.contains(*k))
            })
            .for_each(|(i, asset)| {
                let lower = asset.to_lowercase();
                if let Some(kind) = kinds.iter().position(|k| lower.contains(*k)){
                    match kind {
                        0 => {  // Body
                            let hash = hash_string(asset);
                            if lower.contains("r_c") {
                                if let Some((condition, _)) = get_aid_condition(find_entries_with_model_field(2, asset, |entry, asset| entry.ride_dress_model.is_some_and(|s| s.str_contains(asset))), false){
                                    let name = get_condition_label(&condition);
                                    hashes.add_ride_model(asset);
                                    new_list.add(asset, false, name, 0);
                                }
                            }
                            else if !lower.contains("t_c") && !hashes.body.contains_key(&hash){
                                if let Some((condition, gender)) = get_aid_condition(find_entries_with_model_field(2, asset, |entry, asset| entry.dress_model.is_some_and(|s| s.str_contains(asset))), true){
                                    let name = get_condition_label(&condition);
                                    let o_hash = find_mode_1_body(AssetTableStaticFields::get_condition_index(condition.as_str()), gender).map(|obody|{ hash_string(obody) });
                                    hashes.body.insert(hash, asset.clone());
                                    if condition.starts_with("EID_") && gender != Gender::None && name.is_some() {
                                        new_list.add_engaged_body(name.unwrap(), asset.as_str(), gender == Gender::Female);
                                    }
                                    else if gender != Gender::None {
                                        let female = gender == Gender::Female;
                                        if female { hashes.female_u.push(hash); } else { hashes.male_u.push(hash); }
                                        if let Some(o_hash) = o_hash{
                                            if female { hashes.female_ou.push((hash, o_hash)); }
                                            else { hashes.male_ou.push((hash, o_hash)); }
                                        }
                                        new_list.add(asset, female, name, 0);
                                    }
                                    else {
                                        if male.iter().any(|&s| lower.contains(s)) {
                                            hashes.male_u.push(hash);
                                            if let Some(o_hash) = o_hash{ hashes.male_ou.push((hash, o_hash)); }
                                            new_list.add(asset.as_str(),false, None::<String>, 0);
                                        }
                                        else if female.iter().any(|&s| lower.contains(s)) {
                                            hashes.female_u.push(hash);
                                            if let Some(o_hash) = o_hash{ hashes.female_ou.push((hash, o_hash)); }
                                            new_list.add(asset.as_str(),true, None::<String>, 0);
                                        }
                                        else {
                                            hashes.male_u.push(hash);
                                            hashes.female_u.push(hash);
                                            new_list.add(asset.as_str(),false, None::<String>, 0);
                                            new_list.add(asset.as_str(),true, None::<String>, 0);
                                        }
                                    }
                                }
                            }
                            remove.push(i);
                        }
                        1 => {  // Head
                            if let Some((condition, gender)) = get_aid_condition(
                                find_entries_with_model_field(2, asset, |entry, asset| entry.head_model.is_some_and(|s| s.str_contains(asset))), false,
                            ){
                                let name = get_asset_name(&condition, gender);
                                hashes.add_head(asset.as_str());
                                new_list.add(asset.as_str(), false, name, 0);
                                remove.push(i);
                            }
                        }
                        2 => {  // Hair (uHair_h)
                            if let Some((condition, gender)) =
                                get_aid_condition(
                                    find_entries_with_model_field(
                                        2,
                                        asset,
                                        |entry, asset| entry.hair_model.is_some_and(|s| s.str_contains(asset))
                                    ),
                                    false,
                                )
                            {
                                remove.push(i);
                                hashes.add_hair(asset.as_str(), None);
                                let name = get_asset_name(&condition, gender);
                                new_list.add(asset.as_str(), false, name, 0);
                            }
                        }
                        6 => {  // Effect
                            hashes.add_acc(asset.as_str(), Some(kind as i32 - 4));
                            new_list.add(asset.as_str(),false, None::<String>, 0);
                        }
                        _ => {  // Accessories
                            if kind == 3 { hashes.add_hair(asset.as_str(), None); }
                            else { hashes.add_acc(asset.as_str(), Some(kind as i32 - 4)); }
                            if let Some((condition, gender)) =
                                get_aid_condition(
                                    find_entries_with_model_field(
                                        2,
                                        asset,
                                        |entry, asset| entry.accessory_list.list.iter().any(|a| a.model.is_some_and(|model| model.str_contains(asset)))),
                                    false
                                )
                            {
                                if kind == 3 {  // uAcc_spine2_HairXXX
                                    hashes.add_hair(asset.as_str(), None);
                                    let name = get_asset_name(&condition, gender);
                                    new_list.add(asset.as_str(), false, name, 0);
                                    remove.push(i);
                                }
                                else {
                                    let name = get_asset_name(&condition, gender);
                                    hashes.add_acc(asset.as_str(), Some(kind as i32 - 4));
                                    new_list.add(asset.as_str(),false, name, 0);
                                    remove.push(i);
                                }
                            }
                            else { new_list.add(asset.as_str(),false, None::<String>, 0); }
                        }
                    }
                };
            });
        remove.iter().rev().for_each(|&pos| { assets.remove(pos); });
        new_list.final_add(&mut hashes);
        let dress = DressData::init(&mut hashes);
        let anims = AnimData::init(&mut assets);
        Self {
            dress, anims,
            list: new_list,
            labels: new_labels,
            hashes,
            item: ItemAsset::init(),
            accessory_conditions: AccessoryConditions::new(),
        }
    }
    pub fn is_monster_class(&self, unit: &Unit) -> bool {
        let hash = unit.job.parent.hash;
        (unit.person.gender == 1 || unit.person.gender == 2) && self.dress.transform.iter().find(|x| x.hash == hash && !x.is_transform).is_some()
    }
    pub fn is_transform_class(&self, unit: &Unit) -> bool {
        let hash = unit.job.parent.hash;
        (unit.person.gender == 1 || unit.person.gender == 2) && self.dress.transform.iter().find(|x| x.hash == hash && x.is_transform).is_some()
    }
    pub fn apply_monster_asset(&self, result: &mut AssetTableResult, unit: &Unit, mode: i32) -> bool {
        if let Some(transform) = self.dress.transform.iter().find(|x| x.hash == unit.job.parent.hash) {
            transform.get_result(mode, unit);
            if let Some(item) = transform.item.and_then(|item| self.item.iter().find(|x| x.hash == item)) {
                item.apply(result);
            }
            true
        }
        else { false }
    }
    pub fn adjust_dress(&self, result: &mut AssetTableResult, unit: &Unit, conditions: &AssetConditions) {
        let job = unit.job.parent.hash;
        let engaged = unit.status.value & UnitStatusField::Engaging != 0;
        let dress_gender = if conditions.mode == 2 { self.get_dress_gender(result.dress_model) } else { self.get_dress_gender(result.body_model) };
        let mount = if engaged { None } else { self.anims.get_mount_type(unit, dress_gender) };
        let transforming = conditions.flags.contains(AssetFlags::CombatTranforming);
        if transforming { AnimData::remove(result, true, true); }
        if let Some(rng) = conditions.random_dress.get_random(unit, GameVariableManager::get_number("G_Random_Seed")){
            self.random_body(result, conditions.mode, rng, dress_gender == Gender::Female);
            if transforming { return; }
        }
        if engaged {
            AnimData::remove(result, true, true);
            if let Some(data) = conditions.engaged.as_ref().and_then(|eid| self.dress.get_engaged_dress(eid.into())){
                if data.asset_id == "リュール" {
                    if !unit.is_hero() { data.apply(result, conditions.mode, dress_gender); }
                    else {
                        let mut body = if conditions.mode == 1 { "o" } else { "u" }.to_string();
                        body += if dress_gender == Gender::Male { "B∂ody_Drg0AM_c003" }
                        else { "Body_Drg0AF_c053"};
                        if conditions.mode == 2 { result.dress_model = body.into(); }
                        else { result.body_model = body.into(); }
                    }
                }
                else { data.apply(result, conditions.mode, dress_gender); }

                return;
            }
        }
        if job == 185671037 {   // Alear Fell Child
            if let Some(d) = self.dress.get_job_dress(unit.job, dress_gender) { d.apply(result, conditions.mode, true, engaged); }
        }
        else if transforming  || (unit.force.is_some_and(|x| x.force_type == 1 || x.force_type == 2) && !conditions.flags.is_generic()){
            if let Some(person_data) = self.dress.get_personal_dress(unit) {
                person_data.apply(result, conditions.mode, unit.job.rank > 0 || unit.level > 20, None, &self.hashes);
                return;
            }
            else if let Some(dress_data) = unit.person.get_job().and_then(|job| self.dress.job.iter().find(|x| x.is_match(dress_gender, job))){
                dress_data.apply(result, conditions.mode, false, false);
                return;
            }
        }
        if transforming  { return; }
        if job != 1443627162 && JobDressData::is_sword_fighter(result, conditions.mode) {
            if let Some(dress_data) = self.dress.job.iter().find(|x| x.is_match(dress_gender, unit.job)) {
                dress_data.apply(result, conditions.mode, conditions.flags.contains(AssetFlags::Corrupted), !engaged);
            }
            else if let Some(person_data) = self.dress.get_personal_dress(unit) {
                person_data.apply(result, conditions.mode, unit.job.rank > 0 || unit.level > 20, mount, &self.hashes);
            }
            if !engaged {
                if let Some(dress_data) = self.dress.job.iter().find(|x| x.hash == unit.job.parent.hash) {
                    dress_data.apply_ride(result, conditions.mode, conditions.flags.contains(AssetFlags::Corrupted));
                }
            }
        }
        // Check for Missing
        if conditions.mode == 2 {
            let hash = result.dress_model.get_hash_code();
            if !self.hashes.body.contains_key(&hash) {
                if let Some(person_data) = self.dress.get_personal_dress(unit) {
                    person_data.apply(result, 2, unit.level > 20 || unit.job.rank > 0, mount, &self.hashes);
                }
                else if let Some(dress_data) = self.dress.get_job_dress(unit.job, dress_gender) {
                    dress_data.apply(result, conditions.mode, conditions.flags.contains(AssetFlags::Corrupted), !engaged);
                }
                else { result.dress_model = format!("uBody_Swd0{}_c000", Mount::None.get_gender_race(dress_gender)).into(); }
            }
            if !result.hair_model.is_null() {   //  Hair Adjustment
                if !result.hair_model.contains("null") {
                    if result.accessory_list.list.iter()
                        .any(|acc| acc.model.is_some_and(|model| model.to_string().contains("Hair"))) {
                        result.hair_model = "uHair_null".into();
                    }
                }
            }
        }
        else {
            let hash = result.body_model.get_hash_code();
            if !self.hashes.o_body.contains_key(&hash) {
                if let Some(person_data) = self.dress.get_personal_dress(unit) {
                    person_data.apply(result, 1, unit.level > 20 || unit.job.rank > 0, mount, &self.hashes);
                }
                else if let Some(dress_data) = self.dress.get_job_dress(unit.job, dress_gender) {
                    dress_data.apply(result, conditions.mode, conditions.flags.contains(AssetFlags::Corrupted), !engaged);
                }
                else { result.body_model = format!("oBody_Swd0{}_c000", Mount::None.get_gender_race(dress_gender)).into(); }
            }
            if !result.body_model.is_null() && !result.head_model.is_null(){ // Female Dragon Child Head Adjustment
                if !result.body_model.str_contains("AF_c051") && result.head_model.str_contains("h050") {
                    result.head_model = "oHair_h051".into();
                }
            }
        }
    }
    pub fn correct_anims(&self, result: &mut AssetTableResult, unit: &Unit, profile_flags: i32, conditions: &AssetConditions){
        let dress_gender = if conditions.mode == 2 { self.get_dress_gender(result.dress_model) } else {
            unit.get_dress_gender() };
        if dress_gender != Gender::Male && dress_gender != Gender::Female { return; }
        let kind_ =
            if conditions.flags.contains(AssetFlags::CombatTranforming) { 9 }
            else if conditions.flags.contains(AssetFlags::Bullet) { 10 }
            else { conditions.kind };
        let mount = if conditions.flags.contains(AssetFlags::CombatTranforming) { Mount::None } else { self.anims.get_mount_type(unit, dress_gender).unwrap_or(Mount::None) };
        if conditions.flags.contains(AssetFlags::AxeStaff) && conditions.mode == 2 {
            if dress_gender == Gender::Male { result.body_anims.add("Enb0AM-Ax1_c000_M".into()); }
            else { result.body_anims.add("Enb0AF-Ax1_c000_M".into()); }
            return;
        }
        if conditions.flags.contains(AssetFlags::Vision) {
            self.anims.set_vision_anims(result, dress_gender, conditions.mode);
            return;
        }
        let engaged = unit.is_engaging();
        if engaged && profile_flags & 256 == 0 {
            if conditions.mode == 2 { self.anims.set_engaged_anim(result, dress_gender, unit.job.style, conditions.kind); }
            else { result.body_anim = Some(AnimData::add_uas_gen_str("UAS_Enb0A", dress_gender)); }
        }
        let no_engaged_anim = profile_flags & 256 != 0;
        if conditions.mode == 2 {
            if conditions.flags.contains(AssetFlags::ClassChange) {
                AnimData::remove(result, true, true);
                let anim = format!("Com0{}-No1_c000_N", Mount::None.get_gender_race(dress_gender));
                result.body_anim = Some(anim.as_str().into());
                result.body_anims.add(anim.into());
            }
            else if conditions.flags.contains(AssetFlags::Dance) { self.anims.set_dance_anim(result, dress_gender); }
            else if conditions.flags.contains(AssetFlags::Ballista){
                let anim = format!("Bat0{}-Bw1_c000_L", Mount::None.get_gender_race(dress_gender));
                result.body_anim = Some(anim.as_str().into());
                result.body_anims.add(anim.into());
                AnimData::remove(result, true, true);
            }
            else if engaged {
                let god_unit = unit.god_link.or(unit.god_unit);
                if unit.person.parent.hash == 258677212 {
                    if let Some(god) = god_unit.as_ref().and_then(|d| self.dress.get_engaged_dress(d.data.asset_id)) { god.apply(result, 2, dress_gender); }
                }
                if no_engaged_anim {
                    if !self.anims.has_anim(result, dress_gender, mount, conditions.mode, kind_) || (unit.person.get_job().is_some_and(|v| v.parent.hash == 499211320) && conditions.kind > 0){
                        result.body_anims.clear();
                        self.anims.set_basic_anims(result, unit, kind_, dress_gender, conditions.flags.contains(AssetFlags::Corrupted), engaged);
                    }
                }
                else { self.anims.set_engaged_anim(result, dress_gender, unit.job.style, kind_); }
            }
            else {
                if !self.anims.has_anim(result, dress_gender, mount, conditions.mode, kind_){
                    result.body_anims.clear();
                    self.anims.set_basic_anims(result, unit, kind_, dress_gender, conditions.flags.contains(AssetFlags::Corrupted), engaged);
                }
            }
        }
        else {
            if engaged && !no_engaged_anim { result.body_anim = Some(AnimData::add_uas_gen_str("UAS_Enb0A", dress_gender)); }
            else if !self.anims.has_uas_anims(result, mount, dress_gender, unit.job) {
                self.anims.set_uas_anims(result, mount, dress_gender, unit.job);
            }
        }
    }
    pub fn assign_random_head_hair(&self, result: &mut AssetTableResult, rng: &Random) {
        let head = self.hashes.head.len();
        let index = rng.get_value(head as i32);
        if let Some(head) = self.hashes.head.iter().nth(index as usize) {
            result.head_model = head.1.into();
            if let Some(skin) = self.list.skin.get(head.0) { skin.set_result_color(result, 2); }
        }
        let index = rng.get_value( self.hashes.hair.len() as i32);
        if let Some(hair) = self.hashes.hair.iter().nth(index as usize) { apply_hair(hair.1, result); }
    }
    pub fn random_body(&self, result: &mut AssetTableResult, mode: i32, rng: &Random, female: bool) {
        let hub = GameUserData::get_sequence() == 4;
        if hub {
            if let Some(body) = if female { &self.hashes.female_u } else { &self.hashes.male_u }
                .get_random_element(rng).and_then(|v| self.hashes.body.get(v))
            {
                result.dress_model = body.into();
            }
        }
        else {
            let set = if female { &self.hashes.female_ou } else { &self.hashes.male_ou};
            let index = rng.get_value(set.len() as i32) as usize;
            if let Some(body) = if mode == 2 { self.hashes.body.get(&set[index].0) } else { self.hashes.o_body.get(&set[index].1) } {
                if mode == 2 { result.dress_model = body.into(); } else { result.body_model = body.into(); }
            }
        }
    }
    pub fn try_get_asset(&self, ty: AssetType, hash: i32) -> Option<&String> {
        match ty {
            AssetType::Body => self.hashes.body.get(&hash),
            AssetType::Head => self.hashes.head.get(&hash),
            AssetType::Hair  => self.hashes.hair.get(&hash),
            AssetType::Acc(_)  => self.hashes.acc.get(&hash),
            AssetType::AOC(_) => self.hashes.aoc.get(&hash),
            AssetType::Mount(_) => self.hashes.mounts.get(&hash),
            AssetType::Voice => self.hashes.voice.get(&hash),
            AssetType::Rig => self.hashes.rigs.get(&hash),
            _ => None
        }
    }
    pub fn try_get_asset_hash<'a>(&self, asset: impl Into<&'a Il2CppString>) -> Option<i32> {
        let asset = asset.into();
        let asset_hash = asset.get_hash_code();
        let s =
        self.hashes.body.iter().find(|b| *b.0 == asset_hash)
            .or_else(|| self.hashes.head.iter().find(|b| *b.0 == asset_hash))
            .or_else(|| self.hashes.hair.iter().find(|b| *b.0 == asset_hash))
            .or_else(|| self.hashes.acc.iter().find(|b| *b.0 == asset_hash))
            .or_else(|| self.hashes.aoc.iter().find(|b| *b.0 == asset_hash))
            .or_else(|| self.hashes.o_body.iter().find(|b| *b.0 == asset_hash))
            .or_else(|| self.hashes.o_acc.iter().find(|b| *b.0 == asset_hash))
            .or_else(|| self.hashes.voice.iter().find(|b| *b.0 == asset_hash))
            .or_else(|| self.hashes.rigs.iter().find(|b| *b.0 == asset_hash))
            .map(|b| *b.0);
        s
    }
    pub fn ubody_exist<'a>(&self, dress_model: impl Into<&'a Il2CppString>) -> bool {
        let hash = dress_model.into().get_hash_code();
        self.hashes.body.contains_key(&hash)
    }
    pub fn get_dress_gender(&self, dress_model: &Il2CppString) -> Gender {
        if dress_model.is_null() { return Gender::None; }
        let hash = dress_model.get_hash_code();
        if dress_model.to_string().starts_with("oBody") {
            if self.hashes.male_ou.iter().any(|b| b.1 == hash) { Gender::Male }
            else if self.hashes.female_ou.iter().any(|b| b.1 == hash) { Gender::Female }
            else { Gender::None }
        }
        else {
            self.get_dress_gender_hash(dress_model.get_hash_code()).unwrap_or(Gender::None)
        }
    }
    pub fn get_dress_gender_hash(&self, hashcode: i32) -> Option<Gender> {
        if self.hashes.male_u.contains(&hashcode) { Some(Gender::Male) }
        else if self.hashes.female_u.contains(&hashcode) { Some(Gender::Female) }
        else { None }
    }
    pub fn get_gender_aoc(&self, hash: i32) -> Gender {
        for x in 0..4 {
            if let Some(gender) = self.get_aoc_gender_hash(x, hash){ return gender }
        }
        Gender::None
    }
    pub fn get_aoc_gender(&self, ty: i32, aoc_anim: &Il2CppString) -> Gender {
        self.get_aoc_gender_hash(ty, aoc_anim.get_hash_code()).unwrap_or(Gender::None)
    }
    pub fn get_aoc_gender_hash(&self, _ty: i32, hashcode: i32) -> Option<Gender> {
        if self.hashes.aoc_m.contains(&hashcode) { Some(Gender::Male) }
        else if self.hashes.aoc_f.contains(&hashcode) { Some(Gender::Female) }
        else { None }
    }
}

pub fn get_remove(data: &mut Vec<String>, asset: &str) -> Option<String> {
    data.iter().position(|s| s.contains(asset)).map(|x| data.remove(x)).map(|str| str.split('/').last().unwrap().to_string() )
}

pub fn get_asset_name(condition: &String, gender: Gender) -> Option<String> {
    AccessoryData::get(condition.as_str()).map(|a| {
        if a.condition_gender == 1 && gender == Gender::Male && a.name_m.is_some() { a.name_m.unwrap().to_string() }
        else if a.condition_gender == 2 && gender == Gender::Female && a.name_f.is_some() { a.name_f.unwrap().to_string() }
        else { a.name.to_string() }
    })
        .or_else(|| PersonData::get(condition.as_str()).and_then(|p| p.name.as_ref() ).map(|name| name.to_string()))
        .or_else(|| JobData::get(condition.as_str()).map(|j| j.name.to_string()))
        .or_else(|| GodData::get(condition.as_str()).map(|g| g.mid.to_string()))
}