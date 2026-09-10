use std::{collections::{HashSet, HashMap}, io::{Cursor, Read}};
use bitflags::Flags;
use engage::{
    app::{
        IGameUserDataMethods, IGodDataMethods, IGodUnit, IJobDataMethods, ISingletonClass_1Methods,
        IStructBase, IStructData_1Methods, IUnitMethods, ResourceManager_2
    },
    Dictionary_2Ext,
    List_1Ext,
    system::collections::generic::IList_1,
    app::{IAccessoryDataMethods, IPersonDataMethods, IRandom_2Methods}
};
use unity::{Cast, system::string::IIl2CppStringMethods};
pub use super::*;

mod color;
mod hashes;
mod item;
pub mod anim;
pub(crate) mod dress;
mod util;
mod list;

pub use color::*;
pub use hashes::*;
pub use item::*;
pub use list::*;

use anim::AnimData;
pub use crate::data::dress::{PersonalDressDataFlags, DressData, JobDressData};
use crate::enums::Mount;
use engage::app::{IBitField32, ItemData, Random_2};
pub const KINDS: [&str; 8] = ["uBody_", "uHead_", "uHair_", "uAcc_spine2_Hair", "uAcc_head_", "uAcc_spine", "uAcc_Eff", "uAcc_shield_"];
pub const NULL: [&str; 4] = ["uBody_null", "uHead_null", "uHair_null", "uAcc_head_null"];
const ASSET_FILENAME: [&str; 6] = ["UAS_", "Item/Acc/", "Unit/Model/", "AOC_", "uRig", "uWep"];
pub struct OutfitData {
    pub hashes: OutfitHashes,
    pub dress: DressData,
    pub item: Vec<ItemAsset>,
    pub anims: AnimData,
    pub list: OutfitLists,
    pub labels: AssetLabelTable,
    pub weapons: WeaponAssets,
}

pub struct WeaponAssets {
    pub weapons: Vec<String>,
    pub bow: Vec<String>,
    pub bow_arrow: Vec<String>,
    pub tome: Vec<String>,
    pub rod: Vec<String>,
}
impl WeaponAssets {
    pub fn init(list: Vec<String>) -> Self {
        let mut weapons = vec![];
        let mut bow = vec![];
        let mut bow_arrow = vec![];
        let mut tome = vec![];
        let mut rod = vec![];
        list.into_iter().for_each(|item| {
            if item.contains("_Mg") { tome.push(item); }
            else if item.contains("_Bw") {
                if item.contains("-Ar") { bow_arrow.push(item); }
                else if item.ends_with("-Bw") { bow.push(item); }
            }
            else if item.contains("Rd") { rod.push(item); }
            else if !item.contains("_Ft") && !item.contains("-Sb") && !item.contains("-Gr") { weapons.push(item); }
        });
        Self { weapons, bow, bow_arrow, tome, rod }
    }
    pub fn get_random(&self, kind: i32) -> Option<&String> {
        match kind {
            1|2|3|5 => { self.weapons.get_random_element(Random_2::get_system()) }
            4 => { self.bow.get_random_element(Random_2::get_system()) }
            6 => { self.tome.get_random_element(Random_2::get_system()) }
            7 => { self.tome.get_random_element(Random_2::get_system()) }
            8 => { self.bow_arrow.get_random_element(Random_2::get_system()) }
            _ => None
        }
    }
}
impl OutfitData {
    pub fn init() -> Self {
        let new_labels = AssetLabelTable::new();
        let mut new_list = OutfitLists::new();
        let mut hashes = OutfitHashes::new();
        AssetTable::get_list().iter().for_each(|e|{
            let voice = e.get_voice();
            if !voice.is_null() {
                let hash = voice.get_hash_code();
                if !hashes.voice.contains_key(&hash) { hashes.voice.insert(hash, voice.to_rust_string()); }
            }
        });
        let mut assets: Vec<(i32, String)> = ResourceManager_2::get_s_files().iter()
            .filter_map(|x| if x.0.is_null() { None } else { Some(x.0.to_rust_string()) })
            .filter(|x| !x.contains("null") && !x.contains("AT_c") && ASSET_FILENAME.iter().any(|s| x.contains(*s)))
            .filter_map(|x| x.split("/").last().map(|x| (hash_string(x), x.to_string())))
            .collect();

        let mut remove_hashes: HashSet<i32> = HashSet::new();
        let rig_ends = ["M1", "M", "F", "F1"];
        hashes.rigs = assets.extract_if(.., |(_, s)|
            !s.contains("Wolf") && !s.contains("Drag") && s.contains("uRig_") && (s.contains("Humn") || rig_ends.iter().any(|x| s.ends_with(*x))))
            .collect();
        let weapons: Vec<String> =
            assets.extract_if(.., |(_, s)| s.contains("uWep") && !s.ends_with("-Gr") && !s.ends_with("-Sb") && !s.ends_with("-Qv")).map(|(_, s)| s).collect();
        let weapons = WeaponAssets::init(weapons);
        hashes.o_hair = assets.extract_if(.., |(_, s)| s.contains("oHair_h") || s.contains("oHair_dummy")).collect();
        hashes.o_body = assets.extract_if(.., |(_, s)| s.contains("oBody_")).collect();
        hashes.o_acc = assets.extract_if(.., |(_, s)| s.contains("oAcc_")).collect();

        let labels = include_str!("../data/outfits_label.txt").lines().collect::<Vec<&str>>();
        let mut data = Cursor::new(include_bytes!("../data/outfits.bin"));
        let mut idx_count: [u8; 2] = [0; 2];
        let mut count: [u8; 1] = [0; 1];
        let mut asset_data: [u8; 10] = [0; 10];
        let mut group_num = 0;
        [&mut new_list.char_m, &mut new_list.char_f, &mut new_list.job_m, &mut new_list.job_f, &mut new_list.aids].iter_mut().for_each(|group|{
            data.read_exact(&mut idx_count).unwrap();
            group_num = idx_count[0];
            for _ in 0..idx_count[1] {
                data.read_exact(&mut idx_count).unwrap();
                let label_idx = u16::from_be_bytes(idx_count) as usize;
                data.read_exact(&mut count).unwrap();
                let mut list = vec![];
                for _ in 0..count[0] {
                    data.read_exact(&mut asset_data).unwrap();
                    if let Some((kind, flags)) = AssetType::from_rel_index(asset_data[8] as i32).zip(AssetItemFlags::from_bits(i32::from_be_bytes(asset_data[4..8].try_into().unwrap()))){
                        let hash = i32::from_be_bytes(asset_data[0..4].try_into().unwrap());
                        if let Some(asset) = assets.iter().find(|v| v.0 == hash) {
                            let female = group_num == 1 || group_num == 3 || flags.contains(AssetItemFlags::Female);
                            hashes.add_hash(&asset.1, hash, kind, female);
                            remove_hashes.insert(hash);
                            list.push(AssetItem{ hash, flags, kind, count: asset_data[9] as i32, });
                        }
                        else if hashes.voice.iter().find(|v| *v.0 == hash).is_some() {
                            list.push(AssetItem{ hash, flags, kind, count: asset_data[9] as i32, });
                        }
                    }
                }
                group.push(AssetGroup{ label: labels.get(label_idx).unwrap(), list, });
            }
        });
        new_list.job_count = (
            new_list.job_m.iter().filter(|x|x.list.iter().any(|x| x.kind == AssetType::Body)).count() as i32,
            new_list.job_f.iter().filter(|x|x.list.iter().any(|x| x.kind == AssetType::Body)).count() as i32
        );
        let mut asset_data: [u8; 12] = [0; 12];
        [&mut new_list.engaged, &mut new_list.other].iter_mut().for_each(|group|{
            data.read_exact(&mut idx_count).unwrap();
            let imax = if idx_count[0] == 5 { 38 } else { 382 };
            for _ in 0..imax {
                data.read_exact(&mut asset_data).unwrap();
                let label_idx = u16::from_be_bytes(asset_data[0..2].try_into().unwrap()) as usize;
                if let Some((kind, flags)) = AssetType::from_rel_index(asset_data[10] as i32).zip(AssetItemFlags::from_bits(i32::from_be_bytes(asset_data[6..10].try_into().unwrap()))){
                    let hash = i32::from_be_bytes(asset_data[2..6].try_into().unwrap());
                    if let Some(asset) = assets.iter().find(|v| v.0 == hash) {
                        let female = flags.contains(AssetItemFlags::Female) || flags.contains(AssetItemFlags::LabelFemale);
                        hashes.add_hash(&asset.1, hash, kind, female);
                        let count = asset_data[11] as i32;
                        let is_mess = flags.contains(AssetItemFlags::LabelMess);
                        let female = flags.contains(AssetItemFlags::LabelFemale);
                        let label = labels.get(label_idx).map(|v| v.to_string()).unwrap();
                        remove_hashes.insert(hash);
                        let asset_name = asset.1.clone();
                        group.push(OtherAssetItem{ label,is_mess, female, asset: AssetItem { count, kind, hash, flags}, asset_name});
                    }
                    else if let Some(asset) = hashes.voice.iter().find(|v| *v.0 == hash) {
                        let count = asset_data[11] as i32;
                        let asset_name = asset.1.clone();
                        let is_mess = flags.contains(AssetItemFlags::LabelMess);
                        let female = flags.contains(AssetItemFlags::LabelFemale);
                        let label = labels.get(label_idx).map(|v| v.to_string()).unwrap();
                        group.push(OtherAssetItem{ label,is_mess, female, asset: AssetItem { count, kind, hash, flags}, asset_name});
                    }
                }
            }
        });
        data.read_exact(&mut idx_count).unwrap();
        let count = u16::from_be_bytes(idx_count) as usize;
        let mut asset_data: [u8; 4] = [0; 4];
        for _ in 0..count {
            data.read_exact(&mut asset_data).unwrap();
            let hash = i32::from_be_bytes(asset_data);
            data.read_exact(&mut asset_data).unwrap();
            let color = asset_data.clone();
            new_list.skin.insert(hash, AssetColor{ values: color});
        }
        for x in 0..3 {
            data.read_exact(&mut idx_count).unwrap();
            let count = u16::from_be_bytes(idx_count) as usize;
            for _ in 0..count {
                data.read_exact(&mut asset_data).unwrap();
                let hash1 = i32::from_be_bytes(asset_data);
                data.read_exact(&mut asset_data).unwrap();
                let hash2 = i32::from_be_bytes(asset_data);
                if x == 0 { hashes.head_hair.insert(hash1, hash2); }
                else if x == 1 { hashes.male_ou.push((hash1, hash2)); }
                else { hashes.female_ou.push((hash1, hash2)) }
                if let Some(body) = assets.iter().find(|h| h.0 == hash1){
                    if body.1.contains("Head") { hashes.head.insert(hash1, body.1.clone()); }
                    else if body.1.contains("Hair") { hashes.hair.insert(hash1, body.1.clone()); }
                }
            }
        }
        assets.retain(|(i, _)| !remove_hashes.contains(&i));
        let kinds = ["ubody_", "uhead_c", "uhair_h", "uacc_spine2_hair", "uacc_head_", "uacc_spine", "uacc_eff", "uacc_shield_", "aoc_info_c"];
        let dic_map: HashMap<i32, String> =
            AssetTable::s_condition_indexes()
                .iter()
                .filter(|(x, _)| !x.is_null())
                .map(|(x, i)| (i, x.to_rust_string())).collect();
        assets.iter().enumerate()
            .filter(|(_, (_, s))|{
                let lower = s.to_lowercase();
                !s.contains("null") && kinds.iter().any(|k| lower.contains(*k))
            })
            .for_each(|(_, (hash, asset))| {
                if let Some(item) = AssetItem::new(asset.as_str(), 0) {
                    match item.kind {
                        AssetType::Body => {
                            let mut o_hash = None;
                            let mut name = None;
                            let mut added = false;
                            if let Some((condition, gender)) = find_condition(2, asset, true, item.kind, &dic_map) {
                                if let Some(cond_idx) = get_condition_index(condition.as_str()) {
                                    name = get_condition_label(&condition);
                                    o_hash = find_mode_1_body(cond_idx, gender).map(|obody|hash_string(obody));
                                    if condition.starts_with("EID_") && gender != engage::app::Gender::none() && name.is_some() {
                                        let female = gender == engage::app::Gender::female();
                                        new_list.add_engaged_body(name.clone().unwrap(), asset.as_str(), gender == engage::app::Gender::female());
                                        hashes.try_add_body_by_hash(*hash, o_hash, asset, female);
                                        added = true;
                                    }
                                    else if gender != engage::app::Gender::none() {
                                        let female = gender == engage::app::Gender::female();
                                        hashes.try_add_body_by_hash(*hash, o_hash, asset, female);
                                        new_list.add(asset, female, name.clone(), 0, true);
                                        added = true;
                                    }
                                }
                            }
                            if !added {
                                let female = asset.contains("F_c") || asset.contains("f_c");
                                hashes.try_add_body_by_hash(*hash, o_hash, asset, female);
                                new_list.add(asset.as_str(), female, name.clone(), 1 << 2, true);
                            }
                        }
                        AssetType::Head => {
                            if let Some((condition, gender)) = find_condition(2, asset, false, item.kind, &dic_map) {
                                if let Some(cond_idx) = get_condition_index(condition.as_str()) {
                                    let name = get_asset_name(&condition, gender);
                                    if let Some(o_hair) = find_mode_1_hair(cond_idx).map(|obody| { hash_string(obody) }) {
                                        hashes.head_hair.insert(*hash, o_hair);
                                    }
                                    hashes.add_head(asset.as_str());
                                    new_list.add(asset.as_str(), false, name, 1 << 28, false);
                                }
                            }
                        }
                        AssetType::Hair => {
                            if let Some((condition, gender)) = find_condition(2, asset, false, item.kind, &dic_map) {
                                if let Some(cond_idx) = get_condition_index(condition.as_str()) {
                                    hashes.add_hair(asset.as_str());
                                    if let Some(o_hair) = find_mode_1_hair(cond_idx).map(|o| { hash_string(o) }) { hashes.head_hair.insert(*hash, o_hair); }
                                    let name = get_asset_name(&condition, gender);
                                    new_list.add(asset.as_str(), false, name, 1 << 28, false);
                                }
                            }
                        }
                        AssetType::AOC(0) => {
                            if let Some((condition, gender)) = find_condition(2, asset, true, item.kind, &dic_map) {
                                let name = get_asset_name(&condition, gender);
                                hashes.add_hash(asset, *hash, item.kind, gender.value == 2);
                                new_list.add(asset.as_str(), gender.value == 2, name, 0, false);
                            }
                        }
                        AssetType::Acc(_) => {
                            if let Some((condition, gender)) = find_condition(2, asset, false, item.kind, &dic_map) {
                                let name = get_asset_name(&condition, gender);
                                hashes.add_acc(asset.as_str());
                                new_list.add(asset.as_str(), false, name, 1 << 28, false);
                            }
                        }
                        AssetType::Mount(_) => {
                            if let Some((condition, _)) = find_condition(2, asset, false, item.kind, &dic_map) {
                                let name = get_condition_label(&condition);
                                hashes.add_ride_model(asset.as_str());
                                new_list.add(asset, false, name, 1 << 28, false);
                            }
                        }
                        _ => {}
                    }
                }
            });
        let dress = DressData::init(&mut hashes);
        let anims = AnimData::init(&mut assets);
        hashes.get_info_anim();
        hashes.create_uo_pairs();
        new_list.add_eye_presets(&new_labels);
        new_list.added.sort_by(|a, b| a.asset_name.cmp(&b.asset_name));
        println!("Finished with Outfit Plugin Data");
        Self {
            dress, anims, hashes, weapons,
            list: new_list,
            labels: new_labels,
            item: ItemAsset::init(),
        }
    }
    pub fn is_monster_class(&self, unit: Unit) -> bool {
        let job = unit.get_job();
        if job.is_null() { false }
        else {
            let gender = unit.get_gender().value;
            let hash = job.hash();
            (gender == 1 || gender == 2) && self.dress.transform.iter().find(|x| x.hash == hash && !x.is_transform).is_some()
        }
    }
    pub fn apply_transformation_asset(&self, result: AssetTable_Result, unit: Unit, item: ItemData, mode: i32) -> bool {
        let job = unit.get_job();
        if job.is_null() { false }
        else {
            let hash = job.hash();
            if let Some(transform) = self.dress.transform.iter().find(|x| x.hash == hash) {
                transform.set_result(mode, unit, item, result);
                true
            }
            else { false }
        }
    }
    pub fn get_combat_transformation(&self, unit: Unit, item: ItemData) -> Option<AssetTable_Result> {
        let job = unit.get_job();
        if job.is_null() { None }
        else {
            let hash = job.hash();
            let data = self.dress.transform.iter().find(|x| x.hash == hash)?;
            Some(data.get_result(2, unit, item))
        }
    }
    pub fn adjust_dress(&self, result: AssetTable_Result, unit: Unit, conditions: &AssetConditions) {
        let job_data = unit.get_job();
        let job = job_data.hash();
        let engaged = unit.is_engaging_2();
        let dress_gender = self.get_dress_gender(get_result_dress_body_model(result, conditions.mode));
        let mount = if engaged { None } else { self.anims.get_mount_type(unit, dress_gender) };
        let transforming = conditions.flags.contains(AssetFlags::CombatTranforming);
        if transforming { AnimData::remove(result, true, true); }
        if let Some(rng) = conditions.random_dress.get_random(unit, GameVariableManager::get_number("G_Random_Seed")){
            self.random_body(result, conditions.mode, rng, dress_gender == engage::app::Gender::female());
            if transforming { return; }
        }
        if conditions.mode == 2 && result.get_body_model().is_null() {
            if dress_gender == engage::app::Gender::female() { result.set_body_model("uRig_HumnM1"); }
            else { result.set_body_model("uRig_HumnF1"); }
        }
        let is_promoted = unit.get_level() > 20 || job_data.get_rank() > 0;
        let ignore_engage = conditions.profile_flag & 256 != 0;
        let job_dress_data = self.dress.job.iter().find(|x| x.is_match(dress_gender, job_data));
        if ignore_engage || !engaged {
            if let Some(dress_data) = job_dress_data.as_ref() {
                dress_data.apply_ride(result, conditions.mode, conditions.flags.contains(AssetFlags::Corrupted));
            }
        }
        else { AnimData::remove(result, true, true); }
        if engaged {
            let god_unit = unit.get_god_unit();
            if !god_unit.is_null(){
                let god_data = god_unit.m_data().get_main_data().hash();
                if let Some(data) = UnitAssetMenuData::get_by_person_data(god_data, false) {
                    let hash = data.profile[0].mount[if dress_gender == engage::app::Gender::female() { 1 } else { 0 } as usize];
                    if conditions.mode == 2 {
                        if let Some(body) = self.try_get_asset(AssetType::Body, hash) {
                            set_result_dress_body_model(result, conditions.mode, body.as_str());
                            return;
                        }
                    } 
                    else if conditions.mode == 1 {
                        if let Some(obody) = self.hashes.get_obody(hash) {
                            set_result_dress_body_model(result, conditions.mode, obody);
                            return;
                        }
                    }
                }
            }
            if let Some(data) = conditions.engaged.as_ref().and_then(|eid| self.dress.get_engaged_dress(eid.as_str().into())){
                if data.asset_id == "リュール" {
                    if !unit.is_hero() { data.apply(result, conditions.mode, dress_gender); }
                    else {
                        let mut body = if conditions.mode == 1 { "o" } else { "u" }.to_string();
                        body += if dress_gender == engage::app::Gender::male() { "Body_Drg0AM_c003" } else { "Body_Drg0AF_c053"};
                        set_result_dress_body_model(result, conditions.mode, body);
                    }
                }
                else { data.apply(result, conditions.mode, dress_gender); }
                return;
            }
        }
        else {
            if job == 185671037 {   // Alear Fell Child
                if let Some(d) = job_dress_data.as_ref() { d.apply(result, conditions.mode, true, engaged); }
            }
            else if unit.get_person().get_flag().m_value() & 512 == 0 && !unit.get_person().get_job().is_null() {
                let force = unit.get_force_type();
                if transforming || ((force.value == 1 || force.value == 2) && !conditions.flags.is_generic() && !engaged) {
                    if let Some(person_data) = self.dress.get_personal_dress(unit) {
                        person_data.apply(result, conditions.mode, is_promoted, None, &self.hashes);
                        return;
                    }
                    else if let Some(dress_data) = job_dress_data.as_ref(){
                        dress_data.apply(result, conditions.mode, false, !engaged);
                        return;
                    }
                }
            }
            if transforming { return; }
            if job != 1443627162 && JobDressData::is_sword_fighter(result, conditions.mode) {
                if let Some(dress_data) = job_dress_data.as_ref() {
                    dress_data.apply(result, conditions.mode, conditions.flags.contains(AssetFlags::Corrupted), !engaged);
                }
                else if let Some(person_data) = self.dress.get_personal_dress(unit) {
                    person_data.apply(result, conditions.mode, is_promoted, mount, &self.hashes);
                }
            }
        }
        // Check for Missing
        if conditions.mode == 2 {
            let dress_model = result.get_dress_model();
            let hash = if dress_model.is_null() { 0 } else { dress_model.get_hash_code() };
            if !self.hashes.body.contains_key(&hash) {
                if let Some(person_data) = self.dress.get_personal_dress(unit) {
                    person_data.apply(result, 2, is_promoted, mount, &self.hashes);
                }
                else if let Some(dress_data) = self.dress.get_job_dress(job_data, dress_gender) {
                    dress_data.apply(result, conditions.mode, conditions.flags.contains(AssetFlags::Corrupted), !engaged);
                }
                else { result.set_dress_model(format!("uBody_Swd0{}_c000", Mount::None.get_gender_race(dress_gender))); }
            }
            let hair = result.get_hair_model();
            if !hair.is_null() {   //  Hair Adjustment
                if !hair.contains("null"){
                    if result.m_accessories().items().iter()
                        .filter(|x| !x.is_null() )
                        .any(|x|{
                            let model = x.get_model();
                            if !model.is_null() { model.contains("Hair") } else { false }
                        })
                    {
                        result.set_hair_model("uHair_null");
                    }
                }
            }
        }
        else {
            let body_model = result.get_body_model();
            let hash = if body_model.is_null() { 0 } else { body_model.get_hash_code() };
            if !self.hashes.o_body.contains_key(&hash) {
                if let Some(person_data) = self.dress.get_personal_dress(unit) {
                    person_data.apply(result, 1, is_promoted, mount, &self.hashes);
                }
                else if let Some(dress_data) = self.dress.get_job_dress(job_data, dress_gender) {
                    dress_data.apply(result, conditions.mode, conditions.flags.contains(AssetFlags::Corrupted), !engaged);
                }
                else { result.set_body_model(format!("oBody_Swd0{}_c000", Mount::None.get_gender_race(dress_gender))); }
            }
            if !result.get_body_model().is_null() && !result.get_head_model().is_null(){
                let body = result.get_body_model().to_rust_string();
                let head = result.get_head_model().to_rust_string();
                if body.contains("AF_c051") && head.contains("h050") { result.set_head_model("oHair_h051"); }
            }
            if let Some(ride) = il2str(result.get_ride_model()) { AnimData::scale_ride(result, Mount::determine_mount(ride)); }
        }
    }
    pub fn correct_anims(&self, result: AssetTable_Result, unit: Unit, profile_flags: i32, conditions: &AssetConditions){
        let no_mount = conditions.flags.intersects(AssetFlags::NoMount);
        if no_mount { AnimData::remove(result, true, true); }
        if conditions.flags.contains(AssetFlags::SSupport) { return; }
        let dress_gender = if conditions.mode == 2 { self.get_dress_gender(get_result_dress_body_model(result, conditions.mode)) } else { unit.get_dress_gender()};
        if dress_gender.value == 0 || dress_gender.value > 2 { return; }
        let kind_ =
            if conditions.flags.contains(AssetFlags::CombatTranforming) { 9 }
            else if conditions.flags.contains(AssetFlags::Bullet) { 10 }
            else { conditions.kind };

        let mount = if no_mount { Mount::None } else { self.anims.get_mount_type(unit, dress_gender).unwrap_or(Mount::None) };

        if conditions.flags.contains(AssetFlags::AxeStaff) && conditions.mode == 2 {
            let anim = format!("Com0{}-No1_c000_N", Mount::None.get_gender_race(dress_gender));
            result.m_body_anims().add(anim.into());
            return;
        }
        if conditions.flags.contains(AssetFlags::Vision) {
            self.anims.set_vision_anims(result, dress_gender, conditions.mode);
            return;
        }
        let engaged = unit.is_engaging_2();
        let job = unit.get_job();
        let no_engaged_anim = profile_flags & 256 != 0;
        if engaged && !no_engaged_anim {
            if conditions.mode == 2 { self.anims.set_engaged_anim(result, dress_gender, job.get_style().value, kind_); }
            else { result.set_body_anim(AnimData::add_uas_gen_str("UAS_Enb0A", dress_gender)); }
        }
        if conditions.mode == 2 {
            if conditions.flags.contains(AssetFlags::ClassChange) {
                let anim = format!("Com0{}-No1_c000_N", Mount::None.get_gender_race(dress_gender));
                result.set_body_anim(anim.as_str());
                result.m_body_anims().add(anim.into());
            }
            else if conditions.flags.contains(AssetFlags::Dance) { self.anims.set_dance_anim(result, dress_gender); }
            else if conditions.flags.contains(AssetFlags::Ballista){
                let anim = format!("Bat0{}-Bw1_c000_L", Mount::None.get_gender_race(dress_gender));
                result.set_body_anim(anim.as_str());
                result.m_body_anims().add(anim.into());
            }
            else if conditions.flags.contains(AssetFlags::DragonStone) || conditions.flags.contains(AssetFlags::CombatTranforming) {
                result.m_body_anims().add(AnimData::get_transforming_anim(engaged && !no_engaged_anim, dress_gender.value == 2).into());
            }
            else if engaged {
                let god_unit = unit.get_god_unit();
                if !god_unit.is_null() {
                    if no_engaged_anim && kind_ < 7 {
                        if !self.anims.has_anim(result, dress_gender, mount, conditions.mode, kind_) || ((unit.get_job().hash() == 499211320) && conditions.kind > 0){
                            result.get_body_anims().clear();
                            self.anims.set_basic_anims(result, unit, kind_, dress_gender, conditions.flags.contains(AssetFlags::Corrupted), engaged && !no_engaged_anim);
                        }
                    }
                    else {
                        AnimData::remove(result, true, true);
                        self.anims.set_engaged_anim(result, dress_gender, job.get_style().value, kind_);
                    }
                }
            }
            else {
                if !unit.get_person().get_job().is_null() {
                    if unit.get_person().get_job().hash() == 499211320 && unit.get_job().hash() != 499211320 { result.get_body_anims().clear(); }
                }
                if !self.anims.has_anim(result, dress_gender, mount, conditions.mode, kind_) {
                    result.get_body_anims().clear();
                    self.anims.set_basic_anims(result, unit, kind_, dress_gender, conditions.flags.contains(AssetFlags::Corrupted), engaged);
                }
            }
        }
        else {
            if engaged && !no_engaged_anim { result.set_body_anim(AnimData::add_uas_gen_str("UAS_Enb0A", dress_gender)); }
            else if !self.anims.has_uas_anims(result, mount, dress_gender, job) { self.anims.set_uas_anims(result, mount, dress_gender, job); }
            if il2str(result.get_ride_model()).is_some_and(|v| v.contains("Fyd0DT")) {
                result.set_map_scale_all(1.5);
                result.set_map_scale_wing(0.25);
            }
        }
    }
    pub fn assign_random_head_hair(&self, result: AssetTable_Result, rng: Random_2) {
        let head = self.hashes.head.len();
        let index = rng.get_value_2(head as i32);
        if let Some(head) = self.hashes.head.iter().nth(index as usize) {
            result.set_head_model(head.1.as_str());
            if let Some(skin) = self.list.skin.get(head.0) { skin.set_result_color(result, 2); }
        }
        let index = rng.get_value_2( self.hashes.hair.len() as i32);
        if let Some(hair) = self.hashes.hair.iter().nth(index as usize) { apply_result_hair(hair.1, result); }
    }
    pub fn random_body(&self, result: AssetTable_Result, mode: i32, rng: Random_2, female: bool) {
        let hub = engage::app::GameUserData::get_instance().get_sequence().value == 4;
        if hub {
            let set = if female { &self.hashes.female_u } else { &self.hashes.male_u };
            if let Some(body) = set.get_random_element(rng).and_then(|v| self.hashes.body.get(v)) {
                result.set_dress_model(body.as_str());
            }
        }
        else {
            let set = if female { &self.hashes.female_ou } else { &self.hashes.male_ou};
            let index = rng.get_value_2(set.len() as i32) as usize;
            if let Some(body) = if mode == 2 { self.hashes.body.get(&set[index].0) } else { self.hashes.o_body.get(&set[index].1) } {
                set_result_dress_body_model(result, mode, body.as_str());
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
    pub fn try_get_asset_hash(&self, asset: impl Into<Il2CppString>) -> Option<i32> {
        let asset = asset.into();
        let asset_hash = asset.get_hash_code();
        self.hashes.body.iter().find(|b| *b.0 == asset_hash)
            .or_else(|| self.hashes.head.iter().find(|b| *b.0 == asset_hash))
            .or_else(|| self.hashes.hair.iter().find(|b| *b.0 == asset_hash))
            .or_else(|| self.hashes.acc.iter().find(|b| *b.0 == asset_hash))
            .or_else(|| self.hashes.aoc.iter().find(|b| *b.0 == asset_hash))
            .or_else(|| self.hashes.o_body.iter().find(|b| *b.0 == asset_hash))
            .or_else(|| self.hashes.o_acc.iter().find(|b| *b.0 == asset_hash))
            .or_else(|| self.hashes.voice.iter().find(|b| *b.0 == asset_hash))
            .or_else(|| self.hashes.rigs.iter().find(|b| *b.0 == asset_hash))
            .map(|b| *b.0)
    }
    pub fn ubody_exist(&self, dress_model: impl Into<Il2CppString>) -> bool {
        let hash = dress_model.into().get_hash_code();
        self.hashes.body.contains_key(&hash)
    }
    pub fn get_dress_gender(&self, dress_model: Il2CppString) -> engage::app::Gender {
        if dress_model.is_null() { return engage::app::Gender::none(); }
        let hash = dress_model.get_hash_code();
        if dress_model.to_rust_string().starts_with("oBody") {
            if self.hashes.male_ou.iter().any(|b| b.1 == hash) { engage::app::Gender::male() }
            else if self.hashes.female_ou.iter().any(|b| b.1 == hash) { engage::app::Gender::female() }
            else { engage::app::Gender::none() }
        }
        else { self.get_dress_gender_hash(dress_model.get_hash_code()).unwrap_or(engage::app::Gender::none()) }
    }
    pub fn get_dress_gender_hash(&self, hashcode: i32) -> Option<engage::app::Gender> {
        if self.hashes.male_u.contains(&hashcode) { Some(engage::app::Gender::male()) }
        else if self.hashes.female_u.contains(&hashcode) { Some(engage::app::Gender::female()) }
        else { None }
    }
    pub fn get_gender_aoc(&self, hash: i32) -> engage::app::Gender {
        for x in 0..4 {
            if let Some(gender) = self.get_aoc_gender_hash(x, hash){ return gender }
        }
        engage::app::Gender::none()
    }
    pub fn get_aoc_gender(&self, ty: i32, aoc_anim: Il2CppString) -> engage::app::Gender {
        if aoc_anim.is_null() { engage::app::Gender::none() }
        else { self.get_aoc_gender_hash(ty, aoc_anim.get_hash_code()).unwrap_or(engage::app::Gender::none()) }
    }
    pub fn get_aoc_gender_hash(&self, _ty: i32, hashcode: i32) -> Option<engage::app::Gender> {
        if self.hashes.aoc_m.contains(&hashcode) { Some(engage::app::Gender::male()) }
        else if self.hashes.aoc_f.contains(&hashcode) { Some(engage::app::Gender::female()) }
        else { None }
    }
}

pub fn get_asset_name(condition: &String, gender: engage::app::Gender) -> Option<String> {
    let accessory = engage::app::AccessoryData::get(condition.as_str().into());
    if !accessory.is_null() {
        if accessory.get_condtion_gender().value == 1 && gender.value == 1 && !accessory.get_name_m().is_null() { Some(accessory.get_name_m().to_string()) }
        else if accessory.get_condtion_gender().value == 2 && gender.value == 2 && !accessory.get_name_f().is_null() { Some(accessory.get_name_f().to_string()) }
        else { Some(accessory.get_name().to_string()) }
    }
    else {
        let person = engage::app::PersonData::get(condition.as_str().into());
        if !person.is_null() {
            if !person.get_name().is_null() { Some(person.get_name().to_string()) }
            else { None }
        }
        else {
            let job_data = engage::app::JobData::get(condition.as_str().into());
            if !job_data.is_null() { Some(job_data.get_name().to_string()) }
            else {
                let god_data = engage::app::GodData::get(condition.as_str().into());
                if !god_data.is_null() { Some(god_data.get_mid().to_string()) } else { None }
            }
        }
    }
}
fn find_condition(mode: i32, model: &str, with_gender: bool, kind: AssetType, map: &HashMap<i32, String>) -> Option<(String, engage::app::Gender)> {
    match kind {
        AssetType::Body => {
            let filter = |e: AssetTable, a: &str| il2str(e.get_dress_model()).is_some_and(|s| s == a);
            get_aid_condition(find_entries_with_model_field(mode, model, filter), with_gender, map)
        }
        AssetType::Head => {
            let filter = |e: AssetTable, a: &str| il2str(e.get_head_model()).is_some_and(|s| s == a);
            get_aid_condition(find_entries_with_model_field(mode, model, filter), with_gender, map)
        }
        AssetType::Hair => {
            let filter =
                if model.contains("uHair") { |e: AssetTable, a: &str| il2str(e.get_hair_model()).is_some_and(|s| s == a) }
                else { |e: AssetTable, a: &str| try_find_acc_model_in_entry(e, a).is_some() };
            get_aid_condition(find_entries_with_model_field(mode, model, filter), with_gender, map)
        }
        AssetType::AOC(0) => {
            if model.contains("c7") { None }
            else {
                let filter = |e: AssetTable, a: &str| il2str(e.get_info_anim()).is_some_and(|s| s == a);
                get_jid_condition(find_entries_with_model_field(mode, model, filter), map)
            }
        }
        AssetType::Acc(_) => {
            let filter =  |e: AssetTable, a: &str| try_find_acc_model_in_entry(e, a).is_some();
            get_aid_condition(find_entries_with_model_field(mode, model, filter), with_gender, map)
        }
        AssetType::Mount(_) => {
            let filter = |e: AssetTable, a: &str| il2str(e.get_ride_dress_model()).is_some_and(|s| s == a);
            get_aid_condition(find_entries_with_model_field(mode, model, filter), with_gender, map)
        }
        _ => { None }
    }
}