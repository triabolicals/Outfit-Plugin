use std::collections::{HashMap, HashSet};
use engage::combat::CharacterAppearance;
use engage::gamedata::{Gamedata, GodData, JobData, PersonData, assettable::*};
use engage::gamedata::item::ItemData;
use engage::unit::{Gender, Unit};
use engage::mess::Mess;
use crate::{apply_hair, new_asset_table_accessory, ColorPreset, Mount, OutfitHashes, ACC_LOC};
use crate::data::util::{parse_arg_from_name, AssetTableIndexes};

pub struct DressData {
    pub job: Vec<JobDressData>,
    pub engaged: Vec<EngagedDressData>,
    pub personal: Vec<PersonalDressData>,
    pub transform: Vec<JobTransformData>,
}

impl DressData {
    pub fn init(hashes: &mut OutfitHashes) -> Self {
        let mut job = vec![];
        let mut engaged = vec![];
        let mut section = 0;
        let mut personal = vec![];
        let mut transform_items = vec![];
        let mut result_hashes = HashSet::new();
        let mut mpid_count: HashMap<String, i32> = HashMap::new();
        include_str!("../../data/dress.txt").lines()
            .for_each(|line| {
                if line == "END" { section += 1;}
                else {
                    match section {
                        0 => if let Some(jobs) = JobDressData::from_line(line) { job.extend(jobs); },
                        1 => if let Some(eng) = EngagedDressData::from_line(line) { engaged.push(eng); },
                        2 => {
                            let mut spilt = line.split_whitespace();
                            let job = spilt.next().and_then(|jid| JobData::get(jid));
                            let item = spilt.next().and_then(|iid| ItemData::get(iid));
                            if let Some((job, item)) = job.zip(item){ transform_items.push((job.parent.hash, item.parent.hash)); }
                        }
                        _ => {}
                    }
                }
            });
        let gender = Il2CppArray::new_from_element_class(Il2CppString::class(), 1).unwrap();
        let conditions = ["", "男性", "女装"];

        PersonData::get_list().unwrap().iter().filter(|x| x.gender != 0 && x.get_job().is_some() && x.name.is_some())
            .for_each(|v|{
                let (a, b) = if v.is_hero() || v.flag.value & 128 != 0 { (1, 3) } else { (0, 1) };
                for x in a..b {
                    gender[0] = conditions[x].into();
                    let result = AssetTableResult::get_from_pid(2, v.pid, CharacterAppearance::get_constions(Some(gender)));
                    if let Some(mut person) = PersonalDressData::from_asset_table(result, hashes, v.parent.hash, false){
                        let hash = person.calc_hash();
                        if !result_hashes.contains(&hash) {
                            result_hashes.insert(hash);
                            person.index = v.parent.index;
                            if x > 0 { person.is_female = x == 2; }
                            else { person.generic = v.belong.is_some(); }
                            person.count = if let Some(count) = mpid_count.get_mut(&person.mpid) {
                                *count += 1;
                                *count
                            }
                            else {
                                mpid_count.insert(person.mpid.clone(), 0);
                                0
                            };
                            personal.push(person);
                        }
                    }
                }
            });
        GodData::get_list().unwrap().iter().filter(|x| x.flag.value >= 0 && x.force_type == 0)
            .for_each(|god|{
                let (a, b) = if god.is_hero() { (1, 3) }
                else { (0, 1) };
                for x in a..b {
                    gender[0] = conditions[x].into();
                    let result = AssetTableResult::get_from_god_data(2, god, false, CharacterAppearance::get_constions(None));
                    if let Some(mut person) = PersonalDressData::from_asset_table(result, hashes, god.parent.hash, true) {
                        let hash = person.calc_hash();
                        if !result_hashes.contains(&hash) {
                            if god.gid.str_contains("E00") {
                                person.dark = true;
                                person.mpid = format!("MGID_{}", god.ascii_name.unwrap());
                            }
                            result_hashes.insert(hash);
                            person.count =
                                if let Some(count) = mpid_count.get_mut(&person.mpid) {
                                    *count += 1;
                                    *count
                                }
                                else { mpid_count.insert(person.mpid.clone(), 0);0
                            };
                            personal.push(person);
                        }
                    }
                }
            });
        println!("Appearance Count: {}", personal.len());
        let job_list = JobData::get_list().unwrap();
        let mut transform: Vec<JobTransformData> = job_list.iter().flat_map(|j| JobTransformData::from_job(j)).collect();
        transform_items.iter().for_each(|(hash, item)|{
            if let Some(data) = transform.iter_mut().find(|x| x.hash == *hash) { data.item = Some(*item); }
        });
        let sf = AssetTableStaticFields::get();
        let hashes_left = JobData::get_list().unwrap().iter().filter(|j| !job.iter().any(|x| x.hash == j.parent.hash)).map(|j| j.parent.hash).collect::<Vec<i32>>();
        hashes_left.iter().for_each(|&hash|{
            if let Some(job_data) = JobData::try_get_hash(hash) {
                let condition = AssetTableStaticFields::get_condition_index(job_data.jid);
                if condition >= 0 {
                    let mut mode_1m = None;
                    let mut mode_1f = None;
                    let mut mode_2m = None;
                    let mut mode_2f = None;
                    let mut mode_1r = None;
                    let mut mode_2r = None;
                    sf.search_lists[2].iter().filter(|e| e.ride_dress_model.is_some() || e.dress_model.is_some_and(|e| e.str_contains("M_c") || e.str_contains("F_c")))
                        .for_each(|e|{
                            if let Some(ride_dress) = e.ride_dress_model.as_ref() { mode_2r = Some(ride_dress.to_string()); }
                            if let Some(dress) = e.dress_model.as_ref() {
                                let lower = dress.to_string().to_lowercase();
                                if lower.contains("m_c")  { mode_2m = Some(dress.to_string()); }
                                else if lower.contains("f_c") { mode_2f = Some(dress.to_string()); }
                            }
                        });
                    sf.search_lists[1].iter().filter(|e| e.ride_model.is_some() || e.body_model.is_some_and(|e| e.str_contains("M_c") || e.str_contains("F_c")))
                        .for_each(|e|{
                            if let Some(ride_body) = e.ride_model.as_ref() { mode_1r = Some(ride_body.to_string()); }
                            if let Some(body) = e.body_model.as_ref() {
                                let lower = body.to_string().to_lowercase();
                                if lower.contains("m_c")  { mode_1m = Some(body.to_string()); }
                                else if lower.contains("f_c") { mode_1f = Some(body.to_string()); }
                            }
                        });
                    if let Some((mount, gender)) = mode_2m.as_ref().and_then(|s| Mount::determine_gender(s.as_str())) {
                        job.push(
                            JobDressData {
                                hash, mount, gender,
                                hair_color: 0,
                                dress_model: mode_2m.unwrap(),
                                ride_dress: mode_1r.clone(),
                                ride_body: mode_2r.clone(),
                                body_model: mode_1m,
                            }
                        );
                    }
                    if let Some((mount, gender)) = mode_2f.as_ref().and_then(|s| Mount::determine_gender(s.as_str())) {
                        job.push(
                            JobDressData {
                                hash, mount, gender,
                                hair_color: 0,
                                dress_model: mode_2f.unwrap(),
                                ride_dress: mode_1r,
                                ride_body: mode_2r,
                                body_model: mode_1f,
                            });
                    }
                }
            }
        });
        Self { job, engaged, personal, transform}
    }
    pub fn get_engaged_dress(&self, asset: &Il2CppString) -> Option<&EngagedDressData> {
        let mut str = asset.to_string();
        if str.starts_with("EID_") { str = str.trim_start_matches("EID_").to_string(); }
        self.engaged.iter().find(|x| x.asset_id == str)
    }
    pub fn get_job_dress(&self, job: &JobData, gender: Gender) -> Option<&JobDressData> {
        self.job.iter().find(|x| x.is_match(gender, job))
    }
    pub fn get_personal_dress(&self, unit: &Unit) -> Option<&PersonalDressData> {
        let is_female = unit.get_dress_gender() == Gender::Female;
        if unit.person.flag.value & 512 != 0 { self.get_personal_dress_by_name(unit.person.name.unwrap().to_string().as_str(), is_female) }
        else { self.get_personal_dress_by_person(unit.person, is_female) }
    }
    pub fn get_personal_dress_by_person(&self, person: &PersonData, female: bool) -> Option<&PersonalDressData> {
        let is_lueur = person.parent.index == 1 || person.flag.value & 128 != 0;
        self.personal.iter().find(|x| x.hash == person.parent.hash && !x.generic && ((is_lueur && female == x.is_female) || (!is_lueur)))
            .or_else(||
                person.name.map(|m| m.to_string())
                    .and_then(|name|self.personal.iter().find(|x| !x.generic && x.mpid == name && female == x.is_female))
            )
    }
    pub fn get_personal_dress_by_name(&self, name: &str, female: bool) -> Option<&PersonalDressData> {
        self.personal.iter().find(|x| x.is_female == female && x.mpid == name)
    }
}
#[derive(Default)]
pub struct PersonalDressData {
    pub mpid: String,
    pub is_female: bool,
    pub generic: bool,
    pub emblem: bool,
    pub morph: bool,
    pub dark: bool,
    pub ubody: i32,
    pub ubody2: i32,
    pub uhair: i32,
    pub uhead: i32,
    pub color: [i32; 8],
    pub scale: [u16; 19],
    pub mount: Option<(Mount, i32)>,
    pub acc: [i32; 5],
    pub aoc: [i32; 4],
    pub hash: i32,
    pub index: i32,
    pub count: i32,
    pub voice: i32,
    pub engage_hair: i32,
    pub other_hashes: Vec<i32>,
}
impl PersonalDressData {
    pub fn calc_hash(&self) -> i64 {
        let mut hash = (self.uhead as i64) + ((self.uhair as i64) << 2) + ((self.uhead as i64) << 4);
        for x in 4..8 { hash += (self.color[x] << x) as i64; }
        hash
    }
    pub fn process_from_asset_table(&mut self, result: &AssetTableResult, hash_list: &OutfitHashes) -> bool {
        if result.dress_model.is_null() || result.head_model.is_null() { return false; }
        let mut generic_count = 0;
        let ubody;
        let uhead;
        let mut uhair = 0;
        if result.dress_model.str_contains("uBody_Swd0A") && result.dress_model.str_contains("c000") { generic_count += 1; }
        let hash = result.dress_model.get_hash_code();
        if hash_list.body.contains_key(&hash) { ubody = hash; } else { return false; }
        if result.head_model.str_contains("801") || result.head_model.str_contains("851") { generic_count += 1; }
        if result.head_model.str_contains("c7") { self.morph = true; }
        let hash = result.head_model.get_hash_code();
        if hash_list.head.contains_key(&hash) { uhead = hash; } else { return false; }
        if !result.hair_model.is_null() {
            if result.hair_model.str_contains("801") || result.hair_model.str_contains("851") { generic_count += 1; }
            let hash = result.hair_model.get_hash_code();
            if hash_list.hair.contains_key(&hash) {
                uhair = hash;
            }
        }
        if let Some(model) = result.accessory_list.list.iter().find_map(|x| x.model.filter(|x| x.str_contains("_Hair"))){
            let hashscode = model.get_hash_code();
            if hash_list.hair.contains_key(&hashscode) {
                uhair = hashscode;
            }
        }
        if generic_count >= 2 || ubody == 0 || uhair == 0 || uhead == 0 { return false; }
        self.ubody = ubody;
        self.uhead = uhead;
        self.uhair = uhair;
        self.is_female = hash_list.female_u.contains(&ubody);
        for i in 0..5 {
            if let Some(model) = result.accessory_list.list.iter().find(|x| x.locator.is_some_and(|x| x.to_string() == ACC_LOC[i])).and_then(|v| v.model) {
                let hash = model.get_hash_code();
                if hash_list.acc.contains_key(&hash) { self.acc[i] = hash; }
            }
        }
        for x in 0..8 {
            let colors = [result.unity_colors[x].r , result.unity_colors[x].g, result.unity_colors[x].b];
            let mut converted = 0;
            for y in 0..3 {
                let c = if colors[y] >= 1.0 { 255 }
                else if colors[y] <= 0.0 { 0 }
                else { (colors[y] * 255.0) as i32 };
                converted |= c << y*8;
            }
            self.color[x] = converted;
        }
        for x in 0..16 {
            let v = (result.scale_stuff[x] * 100.0) as u16;
            self.scale[x] = v;
        }
        if let Some(ride_dress) = result.ride_dress_model.as_ref() {
            let hash = ride_dress.get_hash_code();
            let mount = Mount::from(ride_dress.to_string().as_str());
            self.mount = Some((mount, hash));
        }
        if let Some(v) = result.info_anims.map(|v| v.get_hash_code()) { self.aoc[0] = v; }
        if let Some(v) = result.talk_anims.map(|v| v.get_hash_code()) { self.aoc[1] = v; }
        if let Some(v) = result.demo_anims.map(|v| v.get_hash_code()) { self.aoc[2] = v; }
        if let Some(v) = result.hub_anims.map(|v| v.get_hash_code()) { self.aoc[3] = v; }
        if let Some(v) = result.sound.voice.map(|v| v.get_hash_code()).filter(|x| hash_list.voice.contains_key(x)){ self.voice = v; }
        true
    }
    pub fn from_asset_table(result: &AssetTableResult, hash_list: &OutfitHashes, hash: i32, emblem: bool) -> Option<PersonalDressData> {
        let mut new = PersonalDressData::default();
        new.hash = hash;
        new.emblem = emblem;
        if emblem { new.mpid = GodData::try_get_hash(hash).map(|v| v.mid.to_string())?; }
        else { new.mpid = PersonData::try_get_hash(hash).and_then(|v| v.name.as_ref()).map(|v| v.to_string())?; }
        if !new.process_from_asset_table(result, &hash_list) { None }
        else { Some(new) }
    }
    pub fn get_menu_name(&self) -> &'static Il2CppString {
        if self.count == 0 { Mess::get(self.mpid.as_str()) }
        else { format!("{} {}", Mess::get(self.mpid.as_str()), self.count + 1).into() }
    }
    pub fn apply(&self, result: &mut AssetTableResult, mode: i32, promoted: bool, mount: Option<Mount>, outfit_hashes: &OutfitHashes) {
        let body_hash = if promoted && self.ubody2 != 0 { self.ubody2 } else { self.ubody };
        if mode == 2 {
            if let Some(ubody) = outfit_hashes.body.get(&body_hash) { result.dress_model = ubody.into(); }
            if let Some(mount) = self.mount.filter(|x| Some(x.0) == mount).and_then(|m| outfit_hashes.mounts.get(&m.1).zip(mount)) {
                result.ride_dress_model = Some(mount.0.into());
                result.ride_model = Some(mount.1.get_default_asset(true).into());
            }
        } else {
            if let Some(obody) = outfit_hashes.get_obody(self.ubody) { result.body_model = obody.into(); }
            if let Some(mount) = self.mount.filter(|x| Some(x.0) == mount).map(|v| outfit_hashes.get_mount_obody(v.1)) {
                result.ride_model = mount;
            }
        }
    }
    pub fn get_name(&self) -> &'static Il2CppString {
        if self.mpid.len() > 3 { Mess::get(self.mpid.as_str()) }
        else { "Unk".into() }
    }
    pub fn apply_appearance(&self, result: &mut AssetTableResult, mode: i32, promoted: bool, mount: Option<Mount>, outfit_hashes: &OutfitHashes, remove_empty_acc: bool) {
        self.apply(result, mode, promoted, mount, outfit_hashes);
        if mode == 2 {
            if let Some(uhead) = outfit_hashes.head.get(&self.uhead) { result.head_model = uhead.into(); }
            if let Some(uhair) = outfit_hashes.hair.get(&self.uhair) { apply_hair(uhair, result); }
            for x in 0..4 {
                if let Some(acc) = outfit_hashes.acc.get(&self.acc[x]){
                    result.commit_accessory(&new_asset_table_accessory(acc, ACC_LOC[x]));
                }
                else if remove_empty_acc {
                    result.commit_accessory(&new_asset_table_accessory("null", ACC_LOC[x]));
                }
            }
            result.commit_accessory(&new_asset_table_accessory("null", ACC_LOC[4]));
            for x in 0..16 {
                let v = self.scale[x];
                if v > 0 { result.scale_stuff[x] = self.scale[x] as f32 / 100.0; }
            }
        }
        else {
            if let Some(ohair) = outfit_hashes.get_ohair(self.uhair).or_else(|| outfit_hashes.get_ohair(self.uhead)){ result.head_model = ohair.into(); }
            else if self.uhair != 0 { result.head_model = if self.is_female { "oHair_h850" } else { "oHair_h800" }.into() }
            for x in 0..4 {
                if let Some(acc) = outfit_hashes.get_oacc(self.acc[x]){
                    result.commit_accessory(&new_asset_table_accessory(acc, ACC_LOC[x]));
                }
            }
        }
        for x in 0..8 {
            let color = self.color[x];
            if color > 0 { ColorPreset::set_color(&mut result.unity_colors[x], self.color[x]); }
        }
        let is_shop = result.hub_anims.is_some_and(|s| s.str_contains("Shop"));
        let end = if is_shop { 3 } else { 4 };
        for x in 0..end {
            if let Some(aoc) = outfit_hashes.aoc.get(&self.aoc[x]).map(|v| v.into()) {
                match x {
                    0 => result.info_anims = Some(aoc),
                    1 => result.talk_anims = Some(aoc),
                    2 => result.demo_anims = Some(aoc),
                    3 => result.hub_anims = Some(aoc),
                    _ => unreachable!(),
                }
            }
        }
        if let Some(voice) = outfit_hashes.voice.get(&self.voice) { result.sound.voice = Some(voice.into()); }
        result.replace(mode);
    }
    pub fn match_unit(&self, unit: &Unit) -> bool {
        let person_hash = unit.person.parent.hash;
        self.other_hashes.contains(&person_hash) || self.hash == person_hash || unit.person.name.is_some_and(|name| name.to_string() == self.mpid)
    }
}
pub struct JobTransformData {
    pub hash: i32,
    pub is_transform: bool,
    pub asset_table: AssetTableIndexes,
    pub item: Option<i32>,
}
impl JobTransformData {
    pub fn is_monster_entry(entry: &AssetTable) -> bool {
        !entry.condition_indexes.has_condition_index(AssetTableStaticFields::get_condition_index("情報")) &&
            !entry.condition_indexes.has_condition_index(AssetTableStaticFields::get_condition_index("詳細")) &&
        Self::check_asset(entry.hair_model) && Self::check_asset(entry.dress_model) && Self::check_asset(entry.ride_dress_model)
    }
    pub fn check_asset(asset: Option<&Il2CppString>) -> bool {
        asset.is_none_or(|a| {
            let a = a.to_string();
            (a.contains("null") || a.contains("T_c")) && (!a.contains("AM") && !a.contains("AF"))
        })
    }
    pub fn from_job(job_data: &JobData) -> Option<JobTransformData> {
        let job_condition = AssetTableStaticFields::get_condition_index(job_data.jid);
        let transform = AssetTableStaticFields::get_condition_index("Transformed");
        let transform2 = AssetTableStaticFields::get_condition_index("竜石");
        let mode_1_trans_con = AssetTableStaticFields::get_condition_index("竜化");
        if job_condition <= 0 { return None; }
        let sf = AssetTableStaticFields::get();
        let mut asset_table = AssetTableIndexes::default();
        let hash = job_data.parent.hash;
        let mut is_transform = sf.search_lists[2].iter().any(|x| x.condition_indexes.has_condition_index(job_condition) && x.condition_indexes.has_condition_index(transform2));
        if transform >= 0 {  // Transformed Condition Exists
            asset_table.mode_2
                .extend(sf.search_lists[2].iter().filter(|x| x.condition_indexes.has_condition_index(job_condition) && x.condition_indexes.has_condition_index(transform)).map(|v| v.parent.index));

            if !asset_table.mode_2.is_empty() { is_transform = true; }
        }
        if is_transform {
            if asset_table.mode_2.is_empty() {
                asset_table.mode_2.extend(
                    sf.search_lists[2].iter()
                        .filter(|x|
                            x.condition_indexes.has_condition_index(job_condition) &&
                                Self::check_asset(x.hair_model) && Self::check_asset(x.dress_model) && Self::check_asset(x.hair_model)
                        )
                        .map(|entry| entry.parent.index)
                );
            }
            asset_table.mode_1.extend(
                sf.search_lists[1].iter()
                    .filter(|x| x.condition_indexes.has_condition_index(job_condition) && (
                        x.condition_indexes.has_condition_index(mode_1_trans_con) || x.condition_indexes.has_condition_index(transform))
                    )
                    .map(|entry| entry.parent.index)
            );
        }
        else if job_data.get_weapon_mask2().value == (1 << 9) && job_data.mask_skills.find_sid("SID_弾丸装備").is_none() {  // Monster Class
            asset_table.mode_2.extend(
                sf.search_lists[2].iter()
                    .filter(|x| x.condition_indexes.has_condition_index(job_condition) && Self::is_monster_entry(x))
                    .map(|entry|{
                        entry.parent.index
                    })
            );
            asset_table.mode_1.extend(
                sf.search_lists[1].iter()
                    .filter(|x| x.condition_indexes.has_condition_index(job_condition) && Self::is_monster_entry(x))
                    .map(|entry|{
                        entry.parent.index
                    })
            );
        }
        if !asset_table.is_empty() {
            Some(Self{ is_transform, hash, asset_table, item: None, }) }
        else { None }

    }
    pub fn get_result(&self, mode: i32, unit: &Unit) -> &'static mut AssetTableResult{
        let result = AssetTableResult::get_for_unit_hub(unit);
        result.clear();
        let conditions = &AssetTableStaticFields::get().condition_flags;
        conditions.clear();
        conditions.add_unit(unit);
        conditions.add_by_key_(unit.job.jid);
        conditions.add_by_key_(unit.person.pid);
        if let Some(name) = unit.person.name.as_ref() { conditions.add_by_key_(name); }
        self.asset_table.apply(result, mode, Some(conditions));
        if let Some(item_asset) = self.item.and_then(|i| AssetTable::try_index_get(i)){
            result.commit_asset_table(item_asset);
        }
        // result.commit_mode(mode);
        result.replace(mode);
        result
    }
}

pub struct JobDressData {
    pub hash: i32,
    pub gender: Gender,
    pub mount: Mount,
    pub dress_model: String,
    pub hair_color: i32,
    pub body_model: Option<String>,
    pub ride_dress: Option<String>,
    pub ride_body: Option<String>,
}
impl JobDressData {
    pub fn is_sword_fighter(result: &AssetTableResult, mode: i32) -> bool {
        let asset = if mode == 2 { &result.dress_model } else { &result.body_model };
        asset.str_contains("Body_Swd0A") && !asset.str_contains("c251")    //No Lapis
    }
    pub fn is_match(&self, gender: Gender, job: &JobData) -> bool { self.gender == gender && job.parent.hash == self.hash }
    pub fn new_generic_gender(hash: i32, gender: Gender, prefix: &str, ride_dress: &Option<String>, ride_body: &Option<String>) -> Self {
        let dress_model = format!("uBody_{}_c000", if gender == Gender::Male { prefix.replace("*", "M") } else { prefix.replace("*", "F") });
        Self {
            hair_color: 0,
            hash, dress_model, gender,
            body_model: None,
            mount: Mount::from(prefix),
            ride_dress: ride_dress.clone(),
            ride_body: ride_body.clone(),
        }
    }

    pub fn new(hash: i32, prefix: &str, ride_dress: &Option<String>, ride_body: &Option<String>, hair_color: i32) -> Self {
        let dress_model = if prefix.len() > 6 { format!("uBody_{}", prefix) } else { format!("uBody_{}_c000", prefix) };
        let gender = if dress_model.contains("M_c") { Gender::Male } else { Gender::Female };
        Self {
            hash, gender, dress_model, hair_color,
            body_model: None,
            mount: Mount::from(prefix),
            ride_dress: ride_dress.clone(),
            ride_body: ride_body.clone(),
        }
    }
    pub fn from_line(line: &str) -> Option<Vec<Self>> {
        let spilt = line.split_whitespace().collect::<Vec<&str>>();
        if spilt.len() < 2 { None }
        else {
            let is_royal = spilt[1].contains("#");
            let mut class: Vec<Self> = vec![];
            let hair_color = parse_arg_from_name(line, "hair")
                .map(|x| ColorPreset::parse_color(x))
                .unwrap_or(0);

            if let Some(hashes) = crate::data::util::get_job_hashes(spilt[0], is_royal) {
                let ride_dress = spilt.iter().find(|x| x.starts_with("ride=")).map(|v| format!("uBody_{}", v.split_once("=").unwrap().1));
                let ride_body = ride_dress.as_ref().and_then(|ride|{
                    spilt.iter().find(|x| x.starts_with("oride="))
                        .map(|v| format!("oBody_{}", v.split_once("=").unwrap().1))
                        .or_else(|| Some(ride.replace("uBody", "oBody")))
                    });
                if is_royal {
                    if let Some((mount, gender)) = Mount::determine_gender(spilt[1]) {
                        class =
                            hashes.iter().enumerate().map(|(i, &hash)|{
                                let dress_model = format!("uBody_{}", if i == 0 { spilt[1].replace("#", "1") } else { spilt[1].replace("#", "0") });
                                Self {
                                    body_model: Some(dress_model.replace("uBody", "oBody")),
                                    hair_color, mount, gender, hash, dress_model,
                                    ride_body: ride_body.clone(),
                                    ride_dress: ride_dress.clone(),
                                }
                            }).collect()
                    }
                }
                else if spilt[1].ends_with("*") {
                    hashes.iter().for_each(|h| {
                        class.push(Self::new_generic_gender(*h, Gender::Male, spilt[1], &ride_dress, &ride_body));
                        class.push(Self::new_generic_gender(*h, Gender::Female, spilt[1], &ride_dress, &ride_body));
                    });
                }
                else {
                    spilt.iter().filter(|x| !x.starts_with("JID_") && !x.contains("=") && x.len() >= 6)
                        .for_each(|prefix| {
                            hashes.iter().for_each(|h| { class.push(Self::new(*h, prefix, &ride_dress, &ride_body, hair_color)); });
                        })
                    }
            }
            if class.len() == 0 { None } else { Some(class) }
        }
    }
    pub fn apply(&self, result: &mut AssetTableResult, mode: i32, is_morph: bool, with_ride: bool) {
        if mode == 2 { result.dress_model = self.dress_model.as_str().into(); }
        else {
            if let Some(body_model) = self.body_model.as_ref() { result.body_model = body_model.into(); }
        }
        if with_ride { self.apply_ride(result, mode, is_morph); }
        if self.hair_color != 0 {
            ColorPreset::set_color(&mut result.unity_colors[0], self.hair_color);
            ColorPreset::set_color(&mut result.unity_colors[1], self.hair_color);
        }
    }
    pub fn apply_ride(&self, result: &mut AssetTableResult, mode: i32, is_morph: bool) {
        if mode == 2 {
            if let Some(ride) = self.ride_dress.as_ref(){
                if !ride.contains("_c") {
                    if is_morph { result.ride_dress_model = Some(format!("{}_c707", ride).into()); }
                    else { result.ride_dress_model = Some(format!("{}_c000", ride).into()); }
                }
                else { result.ride_dress_model = Some(ride.into()); }
                result.ride_model = Some(self.mount.get_default_asset(true).into());
            }
        }
        else {
            if let Some(ride) = self.ride_body.as_ref() {
                if !ride.contains("_c") {
                    if is_morph { result.ride_model = Some(format!("{}_c707", ride).into()); }
                    else { result.ride_model = Some(format!("{}_c000", ride).into()); }
                }
                else { result.ride_model = Some(ride.into()); }
            }
        }
    }
}

pub struct EngagedDressData {
    pub asset_id: String,
    pub body_prefix: String,
    pub hair_color: i32,
    pub hair_grad: i32,
}
impl EngagedDressData {
    pub fn from_line(line: &str) -> Option<EngagedDressData> {
        let mut spilt = line.split_whitespace();
        let asset_id = spilt.next().map(|x| x.to_string())?;
        let body_prefix = spilt.next().map(|v| v.to_string())?;
        let hair_color = spilt.next().map(|v| ColorPreset::parse_color(v))?;
        let hair_grad = spilt.next().map(|v| ColorPreset::parse_color(v))?;
        Some(Self{ asset_id, body_prefix, hair_color, hair_grad, })
    }
    pub fn apply(&self, result: &mut AssetTableResult, mode: i32, gender: Gender) {
        let mut body = String::from(if mode == 2 { "uBody_" } else { "oBody_"});
        if (self.body_prefix.contains("AF_") && gender == Gender::Female) || (self.body_prefix.contains("AM_") && gender == Gender::Male) {
            let body = format!("{}_{}", body, self.body_prefix.as_str());
            if mode == 2 { result.dress_model = body.into() }
            else { result.body_model = body.into(); }
        }
        body.push_str(self.body_prefix.as_str());
        body.push_str(if gender == Gender::Male { "M_c000" } else { "F_c000" });
        if mode == 2 { result.dress_model = body.into(); } else { result.body_model = body.into(); }
        if self.hair_color != 0 { ColorPreset::set_color(&mut result.unity_colors[0], self.hair_color); }
        if self.hair_grad != 0 { ColorPreset::set_color(&mut result.unity_colors[1], self.hair_grad); }
    }
}