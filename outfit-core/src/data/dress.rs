use std::collections::{HashMap, HashSet};
use engage::{
    app::{AssetTable_Modes, AssetTable_Result, IAssetTableMethods, IAssetTable_ConditionFlagsMethods, IAssetTable_Result, IAssetTable_ResultMethods, IBitField32, IGodDataMethods, IJobDataMethods, IPersonDataMethods, ISkillArrayMethods, IStructBase, IStructData_1Methods, IUnitMethods},
    List_1Ext
};
use engage::prelude::Il2CppString;
use unity::{Cast, IlNull};
use unity::system::string::IIl2CppStringMethods;
use crate::{new_asset_table_accessory, ColorPreset, Mount, OutfitHashes, ACC_LOC, data::util::{parse_arg_from_name, AssetTableIndexes}, set_result_dress_body_model, set_color_by_i32, get_result_dress_body_model, apply_result_hair, get_condition_index, il2str, has_condition_index, try_find_accessory_model, try_get_model_at_locator, get_result_color_i32, get_outfit_data};

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
                            /*
                            let mut spilt = line.split_whitespace();
                            let job = spilt.next().and_then(|jid| JobData::get(jid));
                            let item = spilt.next().and_then(|iid| ItemData::get(iid));
                            if let Some((job, item)) = job.zip(item){ transform_items.push((job.parent.hash, item.parent.hash)); }

                             */
                        }
                        _ => {}
                    }
                }
            });
        let gender_con = ["", "男性", "女装"];
        let result = AssetTable_Result::new();
        let conditions = engage::app::AssetTable::s_condition_flags();
        engage::app::PersonData::get_list()
            .iter()
            .filter(|p| p.get_gender().value != 0 && !p.get_job().is_null() && !p.get_name().is_null())
            .for_each(|p|{
                let (a, b) = if p.is_hero() || p.get_flag().m_value() & 128 != 0 { (1, 3) } else { (0, 1) };
                for x in a..b {
                    result.clear();
                    conditions.add_2(p.get_pid());
                    conditions.add_2(p.get_name());
                    conditions.add_7(engage::app::PersonData::null(), p.get_job(), p.get_asset_force());
                    if x > 0 { conditions.add_2(gender_con[x]); }
                    let belong = p.get_belong();
                    if !belong.is_null() { conditions.add_2(belong ); }
                    result.commit(AssetTable_Modes::combat());
                    result.replace(AssetTable_Modes::combat());
                    if let Some(mut person) = PersonalDressData::from_asset_table(result, hashes, p.hash(), false){
                        let hash = person.calc_hash();
                        if !result_hashes.contains(&hash) {
                            result_hashes.insert(hash);
                            person.index = p.index();
                            person.data_hash = p.hash();
                            if x > 0 { person.is_female = x == 2; } else { person.generic = !belong.is_null(); }
                            person.count =
                                if let Some(count) = mpid_count.get_mut(&person.mpid) { *count += 1;*count }
                                else { mpid_count.insert(person.mpid.clone(), 0);0 };
                            personal.push(person);
                        }
                    }
                }
            });
        engage::app::GodData::get_list().iter()
            .for_each(|god|{
                let (a, b) = if god.is_hero() { (1, 3) } else { (0, 1) };
                for x in a..b {
                    result.clear();
                    let gid = god.get_gid();
                    conditions.add_2(gid);
                    conditions.add_2(god.get_mid());
                    let asset = god.get_asset_id();
                    if !asset.is_null() { conditions.add_2(asset); }
                    if x > 0 { conditions.add_2(gender_con[x]); }
                    result.commit(AssetTable_Modes::combat());
                    result.replace(AssetTable_Modes::combat());
                    if let Some(mut person) = PersonalDressData::from_asset_table(result, hashes, god.hash(), true) {
                        let hash = person.calc_hash();
                        if !result_hashes.contains(&hash) {
                            person.data_hash = god.hash();
                            person.emblem = true;
                            if gid.contains("E00") && !god.get_ascii_name().is_null(){
                                person.dark = true;
                                person.mpid = format!("MGID_{}", god.get_ascii_name());
                            }
                            result_hashes.insert(hash);
                            person.count =
                                if let Some(count) = mpid_count.get_mut(&person.mpid) { *count += 1;*count }
                                else { mpid_count.insert(person.mpid.clone(), 0);0 };
                            personal.push(person);
                        }
                    }
                }
            });
        println!("Appearance Count: {}", personal.len());
        let job_list = engage::app::JobData::get_list();
        let mut transform: Vec<JobTransformData> = job_list.iter().flat_map(|j| JobTransformData::from_job(j)).collect();
        engage::app::PersonData::get_list().iter().filter(|p| !p.get_job().is_null() && !p.get_aid().is_null())
            .for_each(|p|{
                let jhash = p.get_job().hash();
                if !transform.iter().any(|c| c.hash == jhash) {
                    if let Some(j) = JobTransformData::from_person(p) { transform.push(j); }
                }
            });
        transform_items.iter().for_each(|(hash, item)|{
            if let Some(data) = transform.iter_mut().find(|x| x.hash == *hash) { data.item = Some(*item); }
        });
        let search_lists = engage::app::AssetTable::s_search_lists();
        let hashes_left = engage::app::JobData::get_list().iter().filter(|j| !job.iter().any(|x| x.hash == j.hash())).map(|j| j.hash()).collect::<Vec<i32>>();
        hashes_left.iter().for_each(|&hash|{
            let job_data = engage::app::JobData::try_get_from_hash(hash);
            if let Some(condition) = get_condition_index(job_data.get_jid()){
                let mut mode_1m = None;
                let mut mode_1f = None;
                let mut mode_2m = None;
                let mut mode_2f = None;
                let mut mode_1r = None;
                let mut mode_2r = None;
                search_lists.get(2).iter()
                    .filter(|e|
                        has_condition_index(*e, condition) &&
                        (!e.get_ride_dress_model().is_null() || il2str(e.get_dress_model()).is_some_and(|v|{ let l = v.to_lowercase();  l.contains("m_c") || v.contains("f_c") }))
                    )
                    .for_each(|e|{
                        if let Some(ride_dress) = il2str(e.get_ride_dress_model()) { mode_2r = Some(ride_dress); }
                        if let Some(dress) = il2str(e.get_dress_model()) {
                            let lower = dress.to_lowercase();
                            if lower.contains("m_c") { mode_2m = Some(dress); }
                            else if lower.contains("f_c") { mode_2f = Some(dress); }
                        }
                    });
                search_lists.get(1).iter()
                    .filter(|e|
                        has_condition_index(*e, condition) &&
                        (!e.get_ride_model().is_null() || il2str(e.get_body_model()).is_some_and(|v|{ let l = v.to_lowercase();  l.contains("m_c") || v.contains("f_c") }))
                    )
                    .for_each(|e| {
                        if let Some(ride) = il2str(e.get_ride_model()) { mode_1r = Some(ride); }
                        if let Some(body) = il2str(e.get_body_model()) {
                            let lower = body.to_lowercase();
                            if lower.contains("m_c")  { mode_1m = Some(body); }
                            else if lower.contains("f_c") { mode_1f = Some(body); }
                        }
                    });
                    let mount = mode_2r.as_ref().map(|s| Mount::determine_mount(s.as_str())).unwrap_or(Mount::None);
                    if let Some((_, gender)) = mode_2m.as_ref().and_then(|s| Mount::determine_gender(s.as_str())) {
                        job.push(
                            JobDressData {
                                hash, mount, gender, hair_color: 0,
                                dress_model: mode_2m.unwrap(),
                                ride_dress: mode_1r.clone(),
                                ride_body: mode_2r.clone(),
                                body_model: mode_1m,
                            }
                        );
                    }
                    if let Some((_, gender)) = mode_2f.as_ref().and_then(|s| Mount::determine_gender(s.as_str())) {
                        job.push(
                            JobDressData {
                                hash, mount, gender, hair_color: 0,
                                dress_model: mode_2f.unwrap(),
                                ride_dress: mode_1r,
                                ride_body: mode_2r,
                                body_model: mode_1f,
                            });
                    }
            }
        });
        Self { job, engaged, personal, transform}
    }
    pub fn get_engaged_dress(&self, asset: unity::Il2CppString) -> Option<&EngagedDressData> {
        if asset.is_null() { None }
        else {
            let mut str = asset.to_string();
            if str.starts_with("EID_") { str = str.trim_start_matches("EID_").to_string(); }
            self.engaged.iter().find(|x| x.asset_id == str)
        }
    }
    pub fn get_job_dress(&self, job: engage::app::JobData, gender: engage::app::Gender) -> Option<&JobDressData> {
        self.job.iter().find(|x| x.is_match(gender, job))
    }
    pub fn get_personal_dress(&self, unit: engage::app::Unit) -> Option<&PersonalDressData> {
        if unit.is_null() || unit.get_person().is_null() { None }
        else {
            let person = unit.get_person();
            let is_female = unit.get_dress_gender() == engage::app::Gender::female();
            if person.get_flag().m_value() & 512 != 0 { self.get_personal_dress_by_name(person.get_name().to_string().as_str(), is_female) }
            else { self.get_personal_dress_by_person(person, is_female) }
        }
    }
    pub fn get_personal_dress_by_person(&self, person: engage::app::PersonData, female: bool) -> Option<&PersonalDressData> {
        let is_lueur = person.index() == 1 || person.get_flag().m_value() & 128 != 0;
        self.personal.iter().find(|x| x.hash == person.hash() && !x.generic && ((is_lueur && female == x.is_female) || (!is_lueur)))
            .or_else(||
                il2str(person.get_name())
                    .and_then(|name|self.personal.iter().find(|x| !x.generic && x.mpid == name && female == x.is_female))
            )
    }
    pub fn get_personal_dress_by_name(&self, name: &str, female: bool) -> Option<&PersonalDressData> {
        self.personal.iter().find(|x| x.is_female == female && x.mpid == name)
    }
}
#[derive(Default, Clone)]
pub struct PersonalDressData {
    pub mpid: String,
    pub data_hash: i32,
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
    pub fn process_from_asset_table(&mut self, result: AssetTable_Result, hash_list: &OutfitHashes) -> bool {
        let (dress, head) = (result.get_dress_model(), result.get_head_model());
        if dress.is_null() || head.is_null() { return false; }
        let (ubody, uhead) = (dress.get_hash_code(), head.get_hash_code());
        if !hash_list.body.contains_key(&ubody) || !hash_list.head.contains_key(&uhead) { return false; }
        let (dress, head) = (dress.to_rust_string(), head.to_rust_string());
        let mut generic_count = 0;
        let mut uhair = 0;
        if dress.contains("uBody_Swd0A") && dress.contains("c000") { generic_count += 1; }
        if head.contains("801") || head.contains("851") { generic_count += 1; }
        if head.contains("c7") { self.morph = true; }
        let hair = result.get_hair_model();
        if !hair.is_null() {
            let h = hair.get_hash_code();
            let hair = hair.to_rust_string();
            if hair.contains("801") || hair.contains("851") { generic_count += 1; }
            if hash_list.hair.contains_key(&h) { uhair = h; }
        }
        if let Some(model) = try_find_accessory_model(result, "_Hair") {
            let h = unity::Il2CppString::from(model).get_hash_code();
            if hash_list.hair.contains_key(&h) { uhair = h }
        }
        if generic_count >= 2 || ubody == 0 || uhead == 0 { return false; }
        self.ubody = ubody;
        self.uhead = uhead;
        self.uhair = uhair;
        self.is_female = hash_list.female_u.contains(&ubody);
        for i in 0..5 {
            if let Some(model) = try_get_model_at_locator(result, ACC_LOC[i]){
                let h = unity::Il2CppString::from(model).get_hash_code();
                if hash_list.acc.contains_key(&h) { self.acc[i] = h; }
            }
        }
        for x in 0..8 { self.color[x] = get_result_color_i32(result, x); }
        for x in 0..16 { self.scale[x] = crate::get_result_scale_u16(result, x); }
        let ride_dress = result.get_ride_dress_model();
        if !ride_dress.is_null() {
            let hash = ride_dress.get_hash_code();
            let mount = Mount::from(ride_dress.to_rust_string());
            self.mount = Some((mount, hash))
        }
        if let Some(v) = crate::try_get_il2cpp_hash(result.m_info_anim()) { self.aoc[0] = v; }
        if let Some(v) = crate::try_get_il2cpp_hash(result.m_talk_anim()) { self.aoc[1] = v; }
        if let Some(v) = crate::try_get_il2cpp_hash(result.m_demo_anim()) { self.aoc[2] = v; }
        if let Some(v) = crate::try_get_il2cpp_hash(result.m_hub_anim()) { self.aoc[3] = v; }
        if let Some(v) = crate::try_get_il2cpp_hash(result.get_sound().voice_id)
            .filter(|x| hash_list.voice.contains_key(x)) { self.voice = v; }
        true
    }
    pub fn from_asset_table(result: AssetTable_Result, hash_list: &OutfitHashes, hash: i32, emblem: bool) -> Option<PersonalDressData> {
        let mut new = PersonalDressData::default();
        new.hash = hash;
        new.emblem = emblem;
        new.mpid =
            if emblem { engage::app::GodData::try_get_from_hash(hash).get_mid() }
            else { engage::app::PersonData::try_get_from_hash(hash).get_name() }.to_rust_string();
        
        if !new.process_from_asset_table(result, &hash_list) { None } else { Some(new) }
    }
    pub fn get_menu_name(&self) -> Il2CppString {
        if self.count == 0 { engage::app::Mess::get(self.mpid.as_str()) }
        else { format!("{} {}", engage::app::Mess::get(self.mpid.as_str()), self.count + 1).into() }
    }
    pub fn apply(&self, result: AssetTable_Result, mode: i32, promoted: bool, mount: Option<Mount>, outfit_hashes: &OutfitHashes) {
        let body_hash = if promoted && self.ubody2 != 0 { self.ubody2 } else { self.ubody };
        if mode == 2 {
            if let Some(ubody) = outfit_hashes.body.get(&body_hash) { result.set_dress_model(ubody.as_str());}
            if let Some(mount) = self.mount.filter(|x| Some(x.0) == mount).and_then(|m| outfit_hashes.mounts.get(&m.1).zip(mount)) {
                result.set_ride_dress_model(mount.0.as_str());
                result.set_ride_model(mount.1.get_default_asset(true));
            }
        }
        else {
            if let Some(obody) = outfit_hashes.get_obody(self.ubody) { result.set_body_model(obody); }
            if let Some(mount) = self.mount.filter(|x| Some(x.0) == mount).and_then(|v| outfit_hashes.get_mount_obody(v.1)) {
                result.set_ride_model(mount);
            }
        }
    }
    pub fn get_name(&self) -> Il2CppString {
        if self.mpid.len() > 3 { engage::app::Mess::get(self.mpid.as_str()) }
        else { "Unk".into() }
    }
    pub fn apply_appearance(&self, result: AssetTable_Result, mode: i32, promoted: bool, mount: Option<Mount>, outfit_hashes: &OutfitHashes, remove_empty_acc: bool) {
        self.apply(result, mode, promoted, mount, outfit_hashes);
        if mode == 2 {
            if let Some(uhead) = outfit_hashes.head.get(&self.uhead) { result.set_head_model(uhead.as_str()); }
            if let Some(uhair) = outfit_hashes.hair.get(&self.uhair) { apply_result_hair(uhair, result); }
            for x in 0..4 {
                if let Some(acc) = outfit_hashes.acc.get(&self.acc[x]){
                    result.commit_8(new_asset_table_accessory(acc.as_str(), ACC_LOC[x]));
                }
                else if remove_empty_acc { result.commit_8(new_asset_table_accessory("null", ACC_LOC[x])); }
            }
            result.commit_8(new_asset_table_accessory("null", ACC_LOC[4]));
            for x in 0..16 {
                let v = self.scale[x];
                if v > 0 { crate::set_result_scale_u16(result, x, v); }
            }
        }
        else {
            if let Some(ohair) = outfit_hashes.get_ohair(self.uhair).or_else(|| outfit_hashes.get_ohair(self.uhead)){ result.set_head_model(ohair); }
            else if self.uhair != 0 { result.set_head_model( if self.is_female { "oHair_h850" } else { "oHair_h800" }); }
            for x in 0..4 {
                if let Some(acc) = outfit_hashes.get_oacc(self.acc[x]){
                    result.commit_8(new_asset_table_accessory(acc.to_rust_string().as_str(), ACC_LOC[x]));
                }
            }
        }
        for x in 0..8 {
            let color = self.color[x];
            if color > 0 { set_color_by_i32(result, x, self.color[x]); }
        }
        let shop = il2str(result.m_hub_anim()).is_some_and(|v| v.contains("Shop"));
        let end = if shop { 3 } else { 4 };
        for x in 0..end {
            if let Some(aoc) = outfit_hashes.aoc.get(&self.aoc[x]).map(|v| v.as_str().into()) {
                match x {
                    0 => result.set_m_info_anim(aoc),
                    1 => result.set_m_talk_anim(aoc),
                    2 => result.set_m_demo_anim(aoc),
                    3 => result.set_m_hub_anim(aoc),
                    _ => unreachable!(),
                }
            }
        }
        if let Some(voice) = outfit_hashes.voice.get(&self.voice) { result.get_sound().voice_id = voice.as_str().into(); }
        result.replace(AssetTable_Modes{ value: mode});
    }
    pub fn match_unit(&self, unit: engage::app::Unit) -> bool {
        let person = unit.get_person();
        if person.is_null() { return false; }
        let person_hash = person.hash();
        self.other_hashes.contains(&person_hash) || self.hash == person_hash || il2str(person.get_name()).is_some_and(|name| name.to_string() == self.mpid)
    }
    pub fn get_by_hash(hash: i32, emblem: bool, female: bool) -> Option<&'static PersonalDressData> {
        if hash == 276380359 {
            if female {

            }
            else {

            }
        }
        else {
            get_outfit_data().dress.personal.iter().find(|x| x.is_female == female && x.data_hash == hash && emblem == x.emblem)
        }

    }
}
pub struct JobTransformData {
    pub hash: i32,
    pub is_transform: bool,
    pub asset_table: AssetTableIndexes,
    pub item: Option<i32>,
}
impl JobTransformData {
    pub fn is_monster_entry(entry: engage::app::AssetTable) -> bool {
        let c1 = get_condition_index("情報").unwrap();
        let c2 = get_condition_index("詳細").unwrap();
        !has_condition_index(entry, c1) && !has_condition_index(entry, c2) &&
        Self::check_asset(entry.get_head_model()) && Self::check_asset(entry.get_head_model()) && Self::check_asset(entry.get_ride_dress_model())
    }
    pub fn check_asset(asset: unity::Il2CppString) -> bool {
        il2str(asset).is_none_or(|a| (a.contains("null") || a.contains("T_c")) && (!a.contains("AM") && !a.contains("AF")))
    }
    pub fn from_person(person: engage::app::PersonData) -> Option<JobTransformData> {
        if person.get_job().is_null() { return None; }
        let pid = il2str(person.get_pid()).filter(|x| x.ends_with("_竜化"))?;
        let aid = il2str(person.get_aid()).filter(|x| x.ends_with("竜化"))?;
        let aid_condition = get_condition_index(aid.as_str())?;
        let search_lists = engage::app::AssetTable::s_search_lists();
        let mut asset_table = AssetTableIndexes::default();
        asset_table.mode_2.extend(search_lists.get(2).iter().filter(|x| has_condition_index(*x, aid_condition)).map(|x| x.index()));
        let mode_1_trans_condition = get_condition_index("竜化").unwrap();
        let conditions = [il2str(person.get_name()).and_then(|v| get_condition_index(v.as_str())), get_condition_index(pid.as_str()), Some(aid_condition)];
        asset_table.mode_1.extend(
            search_lists.get(1).iter()
                .filter(|x| has_condition_index(*x, mode_1_trans_condition) && (conditions.iter().any(|v| v.is_some_and(|i|has_condition_index(*x, i)))))
                .map(|x| x.index())
        );
        if !asset_table.is_empty() {
            println!("Adding transformation from person: {}", engage::app::Mess::get_game_data_name(person.get_pid()));
            Some(Self{ is_transform: true, hash: person.get_job().hash(), asset_table, item: None })
        }
        else { None }
    }
    pub fn from_job(job_data: engage::app::JobData) -> Option<JobTransformData> {
        let job_condition = get_condition_index(job_data.get_jid())?;
        let transform = get_condition_index("Transformed");
        let transform2 =get_condition_index("竜石")?;
        let mode_1_trans_con = get_condition_index("竜化")?;
        let search_lists = engage::app::AssetTable::s_search_lists();
        let mut asset_table = AssetTableIndexes::default();
        let hash = job_data.hash();
        let mut is_transform = search_lists.get(2).iter().any(|x| has_condition_index(x, job_condition) && has_condition_index(x, transform2));
        if let Some(transform) = transform{
            asset_table.mode_2
                .extend(
                    search_lists.get(2).iter().filter(|x| has_condition_index(*x, job_condition) && has_condition_index(*x, transform))
                        .map(|x| x.index())
                );

            if !asset_table.mode_2.is_empty() {
                is_transform = true;
                asset_table.mode_1.extend(
                    search_lists.get(2).iter()
                        .filter(|x| has_condition_index(*x, job_condition) && (has_condition_index(*x, mode_1_trans_con ) || has_condition_index(*x, transform)))
                        .map(|entry| entry.index())
                );
            }
        }
        if is_transform {
            if asset_table.mode_2.is_empty() {
                asset_table.mode_2.extend(
                    search_lists.get(2).iter().filter(|x| has_condition_index(*x, job_condition) &&
                        Self::check_asset(x.get_hair_model()) && Self::check_asset(x.get_dress_model()) && Self::check_asset(x.get_head_model())
                    ).map(|x| x.index())
                )
            }
        }
        else if job_data.get_weapon_mask_2().m_value() == (1 << 9) && job_data.get_mask_skill().find("SID_弾丸装備").is_null(){
            asset_table.mode_2.extend(
                search_lists.get(2).iter()
                    .filter(|x| has_condition_index(*x, job_condition) && Self::is_monster_entry(*x))
                    .map(|x| x.index())
            );
            asset_table.mode_1.extend(
                search_lists.get(1).iter()
                    .filter(|x| has_condition_index(*x, job_condition) && Self::is_monster_entry(*x))
                    .map(|x| x.index())
            );
        }
        if !asset_table.is_empty() {
            println!("Adding trans for Class: {} [monster: {}]", engage::app::Mess::get_game_data_name(job_data.get_jid()), is_transform);
            Some(Self{ is_transform, hash, asset_table, item: None})
        }
        else { None }
    }
    pub fn get_result(&self, mode: i32, unit: engage::app::Unit) -> AssetTable_Result{
        let result = AssetTable_Result::get_for_unit_hub(unit);
        result.clear();
        let conditions = engage::app::AssetTable::s_condition_flags();
        conditions.clear();
        conditions.add_5(unit);
        conditions.add_2(unit.get_job().get_jid());
        conditions.add_2(unit.get_person().get_pid());
        conditions.add_2(unit.get_person().get_name());
        self.asset_table.apply(result, mode, conditions);
        if let Some(item_asset) = self.item.map(|i| engage::app::AssetTable::try_get_4(i)){
            if item_asset.0 { result.commit_4(item_asset.1); }
        }
        result.replace(AssetTable_Modes{ value: mode });
        result
    }
}

pub struct JobDressData {
    pub hash: i32,
    pub gender: engage::app::Gender,
    pub mount: Mount,
    pub dress_model: String,
    pub hair_color: i32,
    pub body_model: Option<String>,
    pub ride_dress: Option<String>,
    pub ride_body: Option<String>,
}
impl JobDressData {
    pub fn is_sword_fighter(result: AssetTable_Result, mode: i32) -> bool {
        let asset = get_result_dress_body_model(result, mode);
        if asset.is_null() { false }
        else {
            let asset = asset.to_rust_string();
            asset.contains("Body_Swd0A") && !asset.contains("c251")
        }
    }
    pub fn is_match(&self, gender: engage::app::Gender, job: engage::app::JobData) -> bool { self.gender == gender && job.hash() == self.hash }
    pub fn new_generic_gender(hash: i32, gender: engage::app::Gender, prefix: &str, ride_dress: &Option<String>, ride_body: &Option<String>) -> Self {
        let dress_model = format!("uBody_{}_c000", if gender == engage::app::Gender::male() { prefix.replace("*", "M") } else { prefix.replace("*", "F") });
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
        let gender = if dress_model.contains("M_c") { engage::app::Gender::male() } else { engage::app::Gender::female() };
        let body_model = if prefix.contains("c") { Some(dress_model.replace("uBody", "oBody")) } else { None };
        Self {
            hash, gender, dress_model, hair_color,
            body_model,
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
                        class.push(Self::new_generic_gender(*h, engage::app::Gender::male(), spilt[1], &ride_dress, &ride_body));
                        class.push(Self::new_generic_gender(*h, engage::app::Gender::female(), spilt[1], &ride_dress, &ride_body));
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
    pub fn apply(&self, result: AssetTable_Result, mode: i32, is_morph: bool, with_ride: bool) {
        if mode == 2 { result.set_dress_model(self.dress_model.as_str()); }
        else { if let Some(body_model) = self.body_model.as_ref() { result.set_body_model(body_model.as_str()); } }

        if with_ride { self.apply_ride(result, mode, is_morph); }
        if self.hair_color != 0 {
            set_color_by_i32(result, 0, self.hair_color);
            set_color_by_i32(result, 1, self.hair_color);
        }
    }
    pub fn apply_ride(&self, result: AssetTable_Result, mode: i32, is_morph: bool) {
        if mode == 2 {
            if let Some(ride) = self.ride_dress.as_ref(){
                if !ride.contains("_c") {
                    let ride_dress_model = if is_morph { format!("{}_c707", ride) } else { format!("{}_c000", ride) };
                    result.set_ride_dress_model(ride_dress_model);
                }
                else { result.set_ride_dress_model(ride.as_str()); }
                result.set_ride_model(self.mount.get_default_asset(true));
            }
        }
        else {
            if let Some(ride) = self.ride_body.as_ref() {
                if !ride.contains("_c") {
                    let ride_model = if is_morph { format!("{}_c707", ride) } else { format!("{}_c000", ride) };
                    result.set_ride_model(ride_model);
                }
                else { result.set_ride_model(ride.as_str()); }
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
    pub fn apply(&self, result: AssetTable_Result, mode: i32, gender: engage::app::Gender) {
        let mut body = String::from(if mode == 2 { "uBody_" } else { "oBody_"});
        if (self.body_prefix.contains("AF_") && gender == engage::app::Gender::female()) || (self.body_prefix.contains("AM_") && gender == engage::app::Gender::male()) {
            let body = format!("{}_{}", body, self.body_prefix.as_str());
            set_result_dress_body_model(result, mode, body);
        }
        else {
            body.push_str(self.body_prefix.as_str());
            body.push_str(if gender == engage::app::Gender::male() { "M_c000" } else { "F_c000" });
            set_result_dress_body_model(result, mode, body);
        }
        if self.hair_color != 0 { set_color_by_i32(result, 0, self.hair_color); }
        if self.hair_grad != 0 { set_color_by_i32(result, 1, self.hair_grad); }
    }
}