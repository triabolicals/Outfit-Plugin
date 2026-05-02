use engage::{
    gamedata::{Gamedata, GodData, JobData, StructBaseFields},
    gamedata::assettable::*,
    gamedata::skill::SkillData,
    unit::{Gender, Unit}
};
use unity::{
    get_generic_class,
    macro_context::Il2CppClassData,
    prelude::Il2CppString
};
use crate::{Mount, StructTemplate, StructTemplateStaticFields};
use crate::assets::{new_asset_table_accessory};

pub const ANIM_KIND: [&str; 11] = ["No1", "Sw1", "Lc1", "Ax1", "Bw1", "Dg1", "Mg1", "Rd1", "Ft1", "No2", "Mg2"];
pub const INF_KIND: [&str; 11] = ["Com0A", "Swd0A", "Lnc0A", "Axe0A", "Bow0A", "Dge0A", "Mag0A", "Com0A", "Rod0A", "Sds0A", "Mcn3A"];
pub const INF_KIND2: [&str; 11] = ["Com0A", "Swd1A", "Lnc1A", "Axe1A", "Axe2A", "Dge0A", "Mag1A", "Com0A", "Rod1A", "Sds0A", "Mcn3A"];

#[unity::class("Combat", "AnimSetDB")]
pub struct AnimSetDB{
    pub parent: StructBaseFields,
    pub name: &'static mut Il2CppString,
    pub atks: [Option<&'static Il2CppString>; 7],
    pub other: [Option<&'static Il2CppString>; 29],
}
impl Gamedata for AnimSetDB {}
pub struct JobAnimSet {
    pub hash: i32,
    pub gender: Gender,
    pub no_magic_tome: bool,
    pub mount: Mount,
    pub anim: Vec<(i32, String)>,
    pub morph: Option<Vec<(i32, String)>>,
    pub mode_1: Option<String>,
    pub mode_1r: Option<String>,
}
impl JobAnimSet {
    pub fn new(hash: i32, gender: Gender) -> Self {
        Self { hash, gender, no_magic_tome: false, mount: Mount::None, anim: vec![], morph: None, mode_1: None, mode_1r: None }
    }
    pub fn is_match(&self, dress_gender: Gender, job: &JobData) -> bool { self.gender == dress_gender && job.parent.hash == self.hash }
    pub fn apply_anim(&self, result: &mut AssetTableResult, mode: i32, kind: i32, is_morph: bool) -> bool {
        let mut r = false;
        if mode == 2 {
            if let Some(anim) = self.anim.iter().find(|x| x.0 == kind) {
                result.body_anims.add(anim.1.as_str().into());
                r = true;
            }
            if is_morph {
                if let Some(anim) = self.morph.as_ref().and_then(|m| m.iter().find(|x| x.0 == kind)){
                    result.body_anims.add(anim.1.as_str().into());
                    r = true;
                }
            }
            if kind == 6 && self.no_magic_tome { result.right_hand = "uWep_Mg00".into(); }
        }
        else {
            if let Some(body) = self.mode_1.as_ref() {
                result.body_anim = Some(body.into());
                result.body_anims.add(body.into());
                r = true;
            }
            result.ride_anim = self.mode_1r.as_ref().map(|a| a.into());
        }
        r
    }
}

pub struct AnimData {
    pub hashes: Vec<i32>,
    pub job_anims: Vec<JobAnimSet>,
    pub uas: Vec<String>,
    pub engage_atk_anim: Vec<EngageAnim>,
}

pub struct EngageAnim {
    pub sid_hash: Vec<i32>,
    pub male_index: Option<i32>,
    pub female_index: Option<i32>,
}
impl EngageAnim {
    pub fn new(god_data: &GodData) -> Option<Self> {
        let skill = god_data.engage_attack.and_then(|v| SkillData::get(v))?;
        let mut sid_hash: Vec<i32> = skill.style_skills.iter().map(|x| x.parent.hash).collect();
        if let Some(linked) = god_data.engage_attack_link.and_then(|v| SkillData::get(v)) {
            sid_hash.extend(linked.style_skills.iter().map(|x| x.parent.hash))
        }
        let asset = [AssetTableStaticFields::get_condition_index(god_data.asset_id), AssetTableStaticFields::get_condition_index("エンゲージ技")];
        let male = AssetTableStaticFields::get_condition_index("男装");
        let female = AssetTableStaticFields::get_condition_index("女装");
        let sf = AssetTableStaticFields::get();

        let male_index = sf.search_lists[2].iter().find(|x| x.condition_indexes.has_condition_index(asset[0]) && x.condition_indexes.has_condition_index(asset[1]) && x.condition_indexes.has_condition_index(male)).map(|e| e.parent.index);
        let female_index = sf.search_lists[2].iter().find(|x| x.condition_indexes.has_condition_index(asset[0]) && x.condition_indexes.has_condition_index(asset[1]) && x.condition_indexes.has_condition_index(female)).map(|e| e.parent.index);

        Some(Self { sid_hash, male_index, female_index})
    }
    pub fn get(&self, gender: Gender) -> Option<&'static AssetTable> {
        if gender == Gender::Male { self.male_index.and_then(|v| AssetTable::try_index_get(v)) }
        else { self.female_index.and_then(|v| AssetTable::try_index_get(v)) }
    }
}
impl AnimData {
    pub fn init(files: &mut Vec<String>) -> Self {
        let list = AnimSetDB::get_list_mut().unwrap();
        let mut count = AnimSetDB::get_count();
        let mut job_anims = Vec::with_capacity(JobData::get_count() as usize);
        let uas = files.extract_if(.., |x| x.contains("UAS_")).collect::<Vec<_>>();
        let anim_str_list: Vec<String> = list.iter().map(|v| v.name.to_string()).collect();
        if let Some(klass) = get_generic_class!(StructTemplate<AnimSetDB>).ok() {
            let sf = klass.get_static_fields_mut::<StructTemplateStaticFields>();
            for xx in ["Wng0E", "Wng1F", "Wng2D", "Cav0B", "Cav1B"] {
                for kind in [("-Lc1", "_L"), ("-Sw1", "_N")]{
                    anim_str_list.iter().filter(|x| x.ends_with(kind.1) && x.starts_with(xx) && x.contains(kind.0))
                        .for_each(|x|{
                            let new_anim = x.replace(kind.0, "-Dg1");
                            if let Some(new) = create_anim_from_copy(&new_anim, x, None){
                                new.parent.index = count;
                                sf.dictionary.key_list.add(new.parent.key);
                                sf.dictionary.index_key.add(new.parent.key, count);
                                list.add(new);
                                count += 1;
                            }
                        });
                }
            }
            let mut section = 0;
            include_str!("../../data/anim.txt").lines()
                .map(|l| l.split_whitespace().collect::<Vec<&str>>())
                .filter(|l| !l.is_empty())
                .for_each(|l| {
                    if l.len() == 1 { if l[0] == "END" { section += 1; } }
                    else {
                        match section {
                            0 => {  // New from 2 Sets
                                if l.len() >= 2 {
                                    if let Some(new) = create_anim_from_copy(l[0], l[1], l.get(2).map(|v| *v)){
                                        new.parent.index = count;
                                        sf.dictionary.key_list.add(new.parent.key);
                                        sf.dictionary.index_key.add(new.parent.key, count);
                                        list.add(new);
                                        count += 1;
                                    }
                                }
                            }
                            1 => {  // New from Existing Set for Kinds
                                if l.len() >= 3 {
                                    let mut iter = l.iter();
                                    let new_prefix = iter.next().unwrap();
                                    let old_prefix = iter.next().unwrap();
                                    while let Some(kind) = iter.next(){
                                        anim_str_list.iter().filter(|x| x.starts_with(old_prefix) && x.contains(kind))
                                            .for_each(|x|{
                                                let new_anim = x.replace(old_prefix, new_prefix);
                                                if let Some(new) = create_anim_from_copy(&new_anim, x, None){
                                                    new.parent.index = count;
                                                    sf.dictionary.key_list.add(new.parent.key);
                                                    sf.dictionary.index_key.add(new.parent.key, count);
                                                    list.add(new);
                                                    count += 1;
                                                }
                                            });
                                    }
                                }
                            }
                            2 => {
                                let no_magic_tome = l.iter().any(|x| *x == "no_tome");
                                let ride = l.iter().find(|x| x.starts_with("ride=")).map(|x| x.replace("ride=", "UAS_"));
                                let mut iter = l.iter();
                                if let Some(hashes) = iter.next()
                                    .map(|jid| if jid.contains(",") { jid.split(",").collect::<Vec<&str>>() } else { vec![*jid] })
                                    .map(|jids| jids.iter().flat_map(|jid| JobData::get(jid).map(|job| job.parent.hash)).collect::<Vec<i32>>())
                                    .filter(|x| x.len() > 0)
                                {
                                    if let Some(arg) = iter.next() {
                                        if arg.ends_with("*") {
                                            let mount = Mount::from(arg);
                                            for gender in [(Gender::Male, "M"), (Gender::Female,"F")]{
                                                let search = arg.replace("*", gender.1);
                                                let mode_1 = uas.iter().find(|x| x.contains(search.as_str()));
                                                let set = arg.replace("*", gender.1);
                                                if let Some(anim) = get_kind_anims(set.as_str(), &anim_str_list, false)
                                                    .or_else(|| get_kind_anims(set.as_str(), &anim_str_list, false))
                                                {
                                                    let morph = get_kind_anims(set.as_str(), &anim_str_list, true);
                                                    hashes.iter().for_each(|&hash|{
                                                        job_anims.push(JobAnimSet { hash, gender: gender.0, no_magic_tome, mount, anim: anim.clone(), mode_1: mode_1.cloned(), mode_1r: ride.clone(), morph: morph.clone() });
                                                    });
                                                }
                                            }
                                        }
                                        else if arg.contains("#") && arg.len() == 6 && hashes.len() == 1 {
                                            let uas1 = arg.replace("#", "1");
                                            let mode_1 = uas.iter().find(|x| x.contains(uas1.as_str()));
                                            let jid = JobData::try_get_hash(hashes[0]).map(|j| j.jid.to_string()).unwrap();
                                            if let Some((mount, gender)) = Mount::determine_gender(arg){
                                                [("", "1"), ("下級", "0"), ("_E", "1")].iter().for_each(|x| {
                                                    if let Some(job) = JobData::get(format!("{}{}", jid, x.0).as_str()) {
                                                        let hash = job.parent.hash;
                                                        let mut anim_set = arg.replace("#", x.1);
                                                        anim_set.push_str("-#");
                                                        if let Some(anim) = get_kind_anims(anim_set.as_str(), &anim_str_list, false){
                                                            let morph = get_kind_anims(anim_set.as_str(), &anim_str_list, true);
                                                            job_anims.push(JobAnimSet { hash, gender, no_magic_tome, mount, anim, mode_1: mode_1.cloned(), mode_1r: ride.clone(), morph });
                                                        }
                                                    }
                                                });
                                            }
                                        }
                                        else  {
                                            let mut search = arg.to_string();
                                            if search.len() > 6 { search.truncate(6); }
                                            let mode_1 = uas.iter().find(|x| x.contains(search.as_str()))
                                                .or_else(||{
                                                    if search.contains("1") { uas.iter().find(|x| x.contains(search.replace("1", "0").as_str())) }
                                                    else if search.contains("0") { uas.iter().find(|x| x.contains(search.replace("0", "1").as_str())) }
                                                    else { None }
                                                });
                                            if let Some((mount, gender)) = Mount::determine_gender(arg) {
                                                let mut anim_search =  arg.to_string();
                                                    if anim_search.len() == 6 { anim_search.push_str("-#"); }
                                                if let Some(anim) = get_kind_anims(anim_search.as_str(), &anim_str_list, false){
                                                    let morph = get_kind_anims(anim_search.as_str(), &anim_str_list, true);
                                                    hashes.iter().for_each(|&hash| {
                                                        job_anims.push(
                                                            JobAnimSet {
                                                                hash, gender, no_magic_tome, mount, anim: anim.clone(), mode_1: mode_1.cloned(), mode_1r: ride.clone(), morph: morph.clone() }
                                                        );
                                                    });
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                });
        }
        let engage_atk_anim = GodData::get_list().unwrap().iter().filter(|x| x.engage_attack.is_some() ).flat_map(|x| EngageAnim::new(x)).collect::<Vec<_>>();
        println!("Engage Anims: {}", engage_atk_anim.len());

        if let Some((dnc0af, male)) = AnimSetDB::get_mut("Dnc0AF-No1_c000_N").zip(AnimSetDB::get_mut("Dnc0AM-No1_c000_N")){
            dnc0af.atks[0] = Some("Enb0AF-No1_c000_Attack1".into());
            for x in 1..male.atks.len() {
                if male.atks[x].is_none() { dnc0af.atks[x] = None; }
                else if male.atks[x].is_some_and(|x| x.to_string() == "null") { dnc0af.atks[x] = Some("null".into()); }
            }

            for x in 0..male.other.len() {
                if male.other[x].is_none() { dnc0af.other[x] = None; }
                else if male.other[x].is_some_and(|x| x.to_string() == "null") { dnc0af.other[x] = Some("null".into()); }
                else if male.other[x].is_some() { dnc0af.other[x] = Some("Rod0AF-Ft1_c000=".into()); }
            }
            [16, 17, 18, 19, 21, 26, 27, 28].iter().for_each(|x|{
                dnc0af.other[*x] =  Some("Rod0AF-Ft1_c000=".into());
            });
        }
        let hashes = list.iter().map(|anim| anim.name.get_hash_code()).collect::<Vec<i32>>();
        let sf = AssetTableStaticFields::get();
        let hashes_left = JobData::get_list().unwrap().iter().filter(|j| !job_anims.iter().any(|x| x.hash == j.parent.hash)).map(|j| j.parent.hash).collect::<Vec<i32>>();
        // Custom Classes!
        let no_weapon_offset = AssetTableStaticFields::get_condition_index("武器無し");
        hashes_left.iter().for_each(|&hash|{
            if let Some(job) = JobData::try_get_hash(hash) {
                let condition = AssetTableStaticFields::get_condition_index(job.jid);
                if condition >= 0 {
                    let mode_1m = sf.search_lists[1].iter()
                        .find(|entry| entry.condition_indexes.has_condition_index(condition) && entry.body_anim.is_some_and(|x| x.to_string().ends_with("M")))
                        .and_then(|b| b.body_anim).map(|b| b.to_string());
                    let mode_1f = sf.search_lists[1].iter().find(|entry| entry.condition_indexes.has_condition_index(condition) && entry.body_anim.is_some_and(|x| x.to_string().ends_with("F")))
                        .and_then(|b| b.body_anim).map(|b| b.to_string());
                    let mut male = JobAnimSet::new(hash, Gender::Male);
                    let mut female = JobAnimSet::new(hash, Gender::Female);
                    male.mode_1 = mode_1m;
                    female.mode_1 = mode_1f;
                    let mode_1r = sf.search_lists[1].iter().find(|entry| entry.condition_indexes.has_condition_index(condition) && entry.ride_anim.is_some()).and_then(|e| e.ride_anim).map(|r| r.to_string());
                    male.mode_1r = mode_1r.clone();
                    female.mode_1r = mode_1r;
                    sf.search_lists[2].iter().filter(|entry| entry.condition_indexes.has_condition_index(condition) && entry.body_anim.is_some())
                        .for_each(|entry| {
                            let anim = entry.body_anim.unwrap().to_string();
                            if let Some((mount, gender)) = Mount::determine_gender(anim.as_str()) {
                                let job_data = if gender == Gender::Female { &mut female } else { &mut male };
                                job_data.mount = mount;
                                if let Some(kind) = (0..9).into_iter().position(|k| entry.condition_indexes.has_condition_index(no_weapon_offset + k)) {
                                    if kind == 6 {
                                        let no_tome = entry.right_hand.is_some_and(|x| x.str_contains("Mg00") || x.str_contains("null"));
                                        job_data.no_magic_tome |= no_tome;
                                    }
                                    if anim.contains("-#") {
                                        let anim2 = anim.split_once("-#").unwrap().0;
                                        let anim2 = format!("{}-{}", anim2, ANIM_KIND[kind]);
                                        if let Some(anim) = anim_str_list.iter().find(|x| x.starts_with(anim2.as_str())) {
                                            if let Some(a) = job_data.anim.iter_mut().find(|a| a.0 == kind as i32) { a.1 = anim.to_string(); }
                                            else { job_data.anim.push((kind as i32, anim.clone())); }
                                        }
                                    }
                                    else if anim_str_list.contains(&anim) { job_data.anim.push((kind as i32, anim)); }
                                    else if let Some(sp) = anim.split_once("-")
                                        .and_then(|x|
                                            anim_str_list.iter().find(|s| s.starts_with(format!("{}-{}", x.0, ANIM_KIND[kind]).as_str()))
                                        )
                                    {
                                        job_data.anim.push((kind as i32, sp.clone()));
                                    }
                                }
                                else if anim.contains("-#") {
                                    if let Some(sp) = get_kind_anims(anim.as_str(), &anim_str_list, false){
                                        sp.iter().for_each(|s| { if !job_data.anim.iter().any(|x| s.0 == x.0) { job_data.anim.push(s.clone()); } });
                                    }
                                }
                                else if let Some(sp) = anim.split_once("-").and_then(|x| get_kind_anims(format!("{}-#", x.0).as_str(), &anim_str_list, false)){
                                    sp.iter().for_each(|s| { if !job_data.anim.iter().any(|x| s.0 == x.0) { job_data.anim.push(s.clone()); } });
                                }
                            }
                        });
                    if male.anim.len() > 0 { job_anims.push(male); }
                    if female.anim.len() > 0 { job_anims.push(female); }
                }
            }
        });
        Self { hashes, job_anims, uas, engage_atk_anim }
    }
    pub fn get_mount_type(&self, unit: &Unit, gender: Gender) -> Option<Mount> {
        let job_hash = unit.job.parent.hash;
        self.job_anims.iter().find(|job| job.hash == job_hash && gender == job.gender)
            .or_else(|| self.job_anims.iter().find(|job| job.hash == job_hash && job.gender != gender))
            .map(|j| j.mount)
    }
    pub fn has_anim(&self, result: &AssetTableResult, gender: Gender, mount: Mount, mode: i32, kind: i32) -> bool {
        let gen_race = mount.get_gender_race(gender);
        if mode == 2 {
            if kind == 0 && mount == Mount::None { return true; }
            result.body_anims.iter()
                .filter(|x|{
                    let x = x.to_string();
                    !x.starts_with("Com0") && x.contains(ANIM_KIND[kind as usize]) && x.contains(gen_race)
                })
                .map(|anim| {
                    let a = anim.to_string();
                    let a =
                        if a.contains("-Bw1_") { if a.ends_with("M") { a.replace("_M", "_L") } else if a.ends_with("N") { a.replace("_N", "_L") } else { a } }
                        else if a.contains("-Mg1_") { if a.ends_with("L") { a.replace("_N", "_M") } else if a.ends_with("N") { a.replace("_N", "_M") } else { a } }
                        else { a };
                    Il2CppString::new(a).get_hash_code()
                })
                .any(|x| self.hashes.contains(&x))
        }
        else {
            result.body_anims.iter()
                .filter(|x| Mount::from(x.to_string()) == mount)
                .map(|anim| anim.to_string().trim_start_matches("UAS_").to_string() )
                .any(|x| self.uas.contains(&x))
        }
    }
    pub fn set_basic_anims(&self, result: &mut AssetTableResult, unit: &Unit, kind: i32, dress_gender: Gender, is_morph: bool, engaged: bool) -> bool {
        let mount = self.get_mount_type(unit, dress_gender).unwrap_or(Mount::None);
        let mount = if (mount != Mount::None && kind > 7) || mount == Mount::None { Mount::None } else { mount.clone() };
        let dress_gender = if mount == Mount::Pegasus { Gender::Female } else { dress_gender };
        self.add_anim_to_result(result, dress_gender, "Com0A", 0, is_morph);
        if mount == Mount::Cav || mount == Mount::Wolf {
            self.add_anim_to_result(result, dress_gender, "Com0B", 0, is_morph);
            self.add_anim_to_result(result, dress_gender, "Com0B", kind, is_morph);
        }
        if mount == Mount::None  {
            Self::remove(result, true, false);
            let set = if unit.job.rank == 0 && unit.level <= 20 { INF_KIND[kind as usize] } else { INF_KIND2[kind as usize] };
            self.add_anim_to_result(result, dress_gender, set, 0, is_morph);
            if kind > 0 { self.add_anim_to_result(result, dress_gender, set, kind, is_morph); }
        }
        else {
            let set = mount.get_default_asset(false);
            self.add_anim_to_result(result, dress_gender, set, 0, is_morph);
            if kind > 0 { self.add_anim_to_result(result, dress_gender, set, kind, is_morph); }
            
            if result.ride_dress_model.is_none_or(|v| Mount::from(v.to_string().as_str()) != mount ) {
                result.ride_model = Some(mount.get_default_asset(true).into());
                result.ride_dress_model = Some(format!("uBody_{}R_c{}", set, if is_morph { "707"} else { "000"}).into());
            }
        }
        if kind > 3 && kind < 10 { result.commit_accessory(new_asset_table_accessory("null", "l_shld_loc")); }
        if engaged { self.set_engaged_anim(result, dress_gender, unit.job.style, kind); }
        if kind == 10 {
            result.right_hand = "uWep_Mg28".into();
            result.commit_accessory(new_asset_table_accessory("uAcc_shield_Mcn3AM", "l_shld_loc"));
        }
        let added_class_anim = self.add_class_anim(result, dress_gender, kind, unit.job, is_morph);
        if !added_class_anim && engaged {
            self.set_engaged_anim(result, dress_gender, unit.job.style, kind);
        }
        result.replace(2);
        true
    }
    pub fn set_engage_atk_anim(&self, result: &mut AssetTableResult, dress_gender: Gender, unit: &Unit) -> i32 {
        if let Some(engage_atk) = unit.get_engage_attack() {
            if engage_atk.sid.str_contains("SID_三級長エンゲージ技＋") {
                let gen = if dress_gender == Gender::Female { "F" } else { "M" };
                result.body_anims.add(format!("Thr2A{}-Ax1_c000_N", gen).into());
                return 1;
            }
            if (dress_gender == Gender::None || dress_gender == Gender::Other) && !engage_atk.sid.str_contains("SID_チキ") {
                if let Some(s) = self.engage_atk_anim.iter().find(|x| x.sid_hash.contains(&engage_atk.parent.hash)).and_then(|x| x.get(Gender::Female)) {
                    result.commit_asset_table(s);
                    return 2;
                }
            }
            else if dress_gender == Gender::Female || dress_gender == Gender::Male{
                if engage_atk.sid.str_contains("SID_チキ") {  // Divine Blessing with Micaiah
                    if let Some(s) = self.engage_atk_anim.get(3).and_then(|x| x.get(dress_gender)) {
                        result.commit_asset_table(s);
                        return 1;
                    }
                }
                else if let Some(s) = self.engage_atk_anim.iter().find(|x| x.sid_hash.contains(&engage_atk.parent.hash)).and_then(|x| x.get(dress_gender)){
                    result.commit_asset_table(s);
                    return 1;
                }
            }
            println!("Failed to get Engage Atk Anim for: {}", engage_atk.name.unwrap());
        }
        0
    }
    pub fn add_class_anim(&self, result: &mut AssetTableResult, dress_gender: Gender, kind: i32, job: &JobData, is_morph: bool) -> bool {
        match kind {
            (0..9) => {
                self.job_anims.iter().find(|x| x.is_match(dress_gender, job)).map(|d| d.apply_anim(result, 2, kind, is_morph)).unwrap_or(false)
            }
            9 => {
                if dress_gender == Gender::Female { result.body_anims.add("Sds0AF-No2_c099_N".into()); } else { result.body_anims.add("Sds0AM-No2_c049_N".into()); }
                true
            }
            10 => {
                self.add_anim_to_result(result, dress_gender, "Mcn3A", kind, is_morph);
                true
            }
            _ => { false }
        }
    }
    pub fn remove(result: &mut AssetTableResult, mount: bool, acc: bool) {
        if mount {
            result.ride_model = None;
            result.ride_dress_model = None;
            result.ride_anim = None;
            result.commit_accessory(new_asset_table_accessory("null", "c_hip_loc"));
        }
        if acc {
            ["l_shld_loc", "l_swdbox_loc", "l_swdgrip_loc"].iter().for_each(|locator|{
                result.commit_accessory(new_asset_table_accessory("null", *locator));
            });
        }
    }
    pub fn has_uas_anims(&self, result: &mut AssetTableResult, mount: Mount, dress_gender: Gender, job: &JobData) -> bool {
        if result.body_anim.is_none() { false }
        else if mount != Mount::None && result.ride_anim.is_none() { false }
        else {
            let obody_anim = match mount {
                Mount::Griffin|Mount::Pegasus|Mount::Wyvern => { "UAS_oBody_F" }
                Mount::Cav | Mount::Wolf => { "UAS_oBody_B" }
                Mount::None => { "UAS_oBody_A" }
            };
            if !result.body_anims.iter().any(|x| x.str_contains(obody_anim)) { false }
            else {
                self.job_anims.iter().find(|x| x.is_match(dress_gender, job))
                    .and_then(|x| x.mode_1.as_ref())
                    .map(|mode_1| mode_1.as_str())
                    .map(|mode_1| result.body_anims.iter().any(|x| x.str_contains(mode_1)))
                    .unwrap_or(false)
            }
        }
    }
    pub fn set_uas_anims(&self, result: &mut AssetTableResult, mount: Mount, dress_gender: Gender, job: &JobData) {
        result.body_anims.clear();
        result.scale_stuff[16] = if mount != Mount::None { 2.1 } else { 2.6 };
        result.scale_stuff[17] = 1.1;
        result.body_anims.add(Self::add_uas_gen_str("UAS_oBody_A", dress_gender));
        match mount {
            Mount::Pegasus | Mount::Wyvern | Mount::Griffin => {
                let anim = Self::add_uas_gen_str("UAS_oBody_F", dress_gender);
                result.body_anims.add(anim);
                result.scale_stuff[18] = if mount == Mount::Wyvern { 0.5 } else { 0.6 };
            }
            Mount::Cav | Mount::Wolf => {
                let anim = Self::add_uas_gen_str("UAS_oBody_B", dress_gender);
                result.body_anims.add(anim);
                result.scale_stuff[16] = 2.4;
            }
            _ => { result.body_anim = Some(Self::add_uas_gen_str("UAS_oBody_A", dress_gender)); }
        }
        if mount != Mount::None && result.ride_model.is_none_or(|ride| Mount::from(ride.to_string().as_str()) != mount){
            result.ride_model = Some(format!("oBody_{}R_c000", mount.get_default_asset(false)).into());
        }
        if let Some(job_data) = self.job_anims.iter().find(|x| x.is_match(dress_gender, job)) {
            job_data.apply_anim(result, 1, 0, false);
        }
        else if mount != Mount::None {
            if result.body_anim.is_none_or(|x|Mount::from(x.to_string().as_str()) != mount) {
                result.body_anim = Some(Self::add_uas_gen_str(format!("UAS_{}", mount.get_default_asset(false)).as_str(), dress_gender));
            }
            if mount != Mount::None && result.ride_anim.is_none_or(|x|Mount::from(x.to_string().as_str()) != mount) {
                result.ride_anim = Some(format!("UAS_{}R", mount.get_default_asset(false)).as_str().into());
            }
        }
    }
    pub fn set_dance_anim(&self, result: &mut AssetTableResult, dress_gender: Gender) {
        result.left_hand = "uWep_Mg00".into();
        result.right_hand = "uWep_Mg00".into();
        Self::remove(result, true, true);
        result.magic = "RD_Dance".into();
        match dress_gender {
            Gender::Female => {
                if result.body_anims.iter().find(|s|{ s.str_contains("AF-No1") && AnimSetDB::get(s.to_string().as_str()).is_some_and(|set| set.atks[0].is_some()) }).is_none() {
                    result.body_anims.add("Dnc0AF-No1_c000_N".into());
                    result.body_anim = Some("Dnc0AF-No1_c000_N".into());
                }
            }
            Gender::Male => { result.body_anims.add("Dnc0AM-No1_c000_N".into()); }
            Gender::Other => { result.body_anims.add("Ent0AT-Ft3_c000_N".into()); }
            _ => {}
        }
    }
    pub fn set_vision_anims(&self, result: &mut AssetTableResult, dress_gender: Gender, mode: i32) {
        Self::remove(result, true, true);
        if mode == 2 {
            result.body_anims.clear();
            self.add_anim_to_result(result, dress_gender, "Com0A", 0, false);
            self.add_anim_to_result(result, dress_gender, "Com0A", 1, false);
            self.add_anim_to_result(result, dress_gender, "Enb0A", 0, false);
            self.add_anim_to_result(result, dress_gender, "Enb0A", 1, false);
        }
        else { result.body_anim = Some(Self::add_uas_gen_str("UAS_Enb0A", dress_gender)); }

    }
    pub fn set_engaged_anim(&self, result: &mut AssetTableResult, gender: Gender, style: i32, kind: i32) {
        result.body_anims.clear();
        let style =
            if kind == 10 { 4 }
            else if kind == 9 { 7 }
            else { style };
        let set =
            match style {
                4 => { "Enh0A" }    // Amr
                5 => { "Enw0A" }    //  Fly
                6 => { "Enm0A" }    // Magic
                7 => { "End0A" }    // Dragon
                _ => { "Enb0A" }    // Backup
            };
        Self::remove(result, true, true);
        self.add_anim_to_result(result, gender, "Com0A", 0, false);
        self.add_anim_to_result(result, gender, set, 0, false);
        if kind == 9 { result.body_anims.add(if gender == Gender::Male { "End0AM-No2_c049_N" } else { "End0AF-No2_c099_N" }.into()); }
        else if kind == 10 { result.body_anims.add(if gender == Gender::Male { "Enh0AM-Mg2_c000_M" } else { "Enh0AF-Mg2_c000_M" }.into()); }
        else { self.add_anim_to_result(result, gender, set, kind, false); }
    }
    fn add_anim_to_result(&self, result: &mut AssetTableResult, gender: Gender, anim_set_prefix: impl AsRef<str> + std::fmt::Display, kind: i32, is_morph: bool){
        let gender_str = if gender == Gender::Male { ("M", "c702") } else { ("F", "c703") };
        let nlm = if kind == 6 || kind == 10 { "M" } else if kind == 4 { "L" } else { "N" };
        if kind != 9 {
            result.body_anims.add(format!("{}{}-{}_c000_{}", anim_set_prefix, gender_str.0, ANIM_KIND[kind as usize], nlm).into());
            if is_morph {
                let morph_set = format!("{}{}-{}_{}_{}", anim_set_prefix, gender_str.0, ANIM_KIND[kind as usize], gender_str.1, nlm);
                result.body_anims.add(morph_set.into());
            }
        }
        else { result.body_anims.add(if gender == Gender::Male { "Sds0AM-No2_c049_N" } else { "Sds0AF-No2_c099_N" }.into()); }
    }
    pub fn add_uas_gen_str(set: &str, gender: Gender) -> &'static Il2CppString {
        let mut s = set.to_string();
        s.push( if gender == Gender::Male { 'M' } else { 'F' });
        s.into()
    }
    pub fn adjust_engage_atk(result: &mut AssetTableResult, gender: Gender) {
        result.body_anims.iter_mut()
            .filter(|x| AnimSetDB::get(x.to_string().as_str()).is_some_and(|x| x.other[25].is_some_and(|x| x.to_string() == "=")))
            .for_each(|x| {
                let anim = x.to_string();
                if anim.contains("F-") && gender == Gender::Male { *x = anim.replace("F-", "M-").into() }
                else if anim.contains("M-") && gender == Gender::Female { *x = anim.replace("M-", "F-").into() }
            });
    }
}
fn create_anim_from_copy<S: AsRef<str>>(new_anim_name: S, copy_anim_name_1: S, copy_anim_name_2: Option<S>) -> Option<&'static mut AnimSetDB> {
    if AnimSetDB::get(new_anim_name.as_ref()).is_some() { None }
    else if let Some(old) = AnimSetDB::get(copy_anim_name_1.as_ref()){
        let new = AnimSetDB::instantiate().unwrap();
        new.parent.key = new_anim_name.as_ref().into();
        new.name = new_anim_name.as_ref().into();
        let src_anim = copy_anim_name_1.as_ref().to_string();
        let trimmed = &src_anim[0..src_anim.len() - 2];
        new.atks.iter_mut().zip(old.atks.iter()).for_each(|(i, c)| { *i = c.as_ref().map(|i| get_anim(i, trimmed)); });
        if let Some(old2) = copy_anim_name_2.and_then(|o|  AnimSetDB::get(o.as_ref())){
            let src_anim = copy_anim_name_1.as_ref().to_string();
            let trimmed = &src_anim[0..src_anim.len() - 2];
            new.other.iter_mut().zip(old2.other.iter()).for_each(|(i, c)| { *i = c.as_ref().map(|i| get_anim(i, trimmed)); });
        }
        else { new.other.iter_mut().zip(old.other.iter()).for_each(|(i, c)| { *i = c.as_ref().map(|i| get_anim(i, trimmed)); }); }
        for x in 8..11 { new.other[x] = None; }
        Some(new)
    }
    else { None }
}
fn get_kind_anims(anim_set: &str, anim_list: &Vec<String>, morph: bool) -> Option<Vec<(i32, String)>>{
    let mut result = Vec::new();
    ANIM_KIND.iter().enumerate().for_each(|(kind, ty)|{
        let search = anim_set.replace("#",*ty);
        if let Some(anim) = anim_list.iter().find(|x| x.contains(search.as_str()) && x.contains(ty) && morph == x.contains("_c70")){
            result.push((kind as i32, anim.clone()));
        }
    });
    if result.len() > 0 { Some(result) }
    else { None }
}
fn get_anim(anim: &Il2CppString, set: &str) -> &'static Il2CppString {
    let anim = anim.to_string();
    if anim == "=" { format!("{}=", set).into() }
    else if anim.starts_with("=") && anim.len() > 2 { anim.replace("=", format!("{}_", set).as_str()).into() }
    else { anim.as_str().into() }
}
