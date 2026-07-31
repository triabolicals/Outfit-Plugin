use std::collections::{HashMap, HashSet};
use bitflags::{bitflags, Flags};
use engage::{
    app::{FaceThumbnail, GodData, ISpriteAtlasManager_2Methods, PersonData, ResourceManager_2, TelopManager},
    app::{AssetTable_Modes, AssetTable_Result, IAssetTableMethods, IAssetTable_ConditionFlagsMethods, IAssetTable_Result, IAssetTable_ResultMethods, IBitField32, IGodDataMethods, IJobDataMethods, IPersonDataMethods, ISkillArrayMethods, IStructBase, IStructData_1Methods, IUnitMethods},
    Dictionary_2Ext,
    List_1Ext,
    combat::{CharacterAppearance},
    prelude::Il2CppString,
    unity_engine::Sprite
};
use engage::app::{AssetTable, IJobData, IUnit, IUnitEdit, IUnitEditMethods, JobData, Mess};
use unity::{Cast};
use unity::system::string::IIl2CppStringMethods;
use crate::{new_asset_table_accessory, ColorPreset, Mount, OutfitHashes, ACC_LOC, data::util::{parse_arg_from_name, AssetTableIndexes}, set_result_dress_body_model, set_color_by_i32, get_result_dress_body_model, apply_result_hair, get_condition_index, il2str, has_condition_index, try_find_accessory_model, try_get_model_at_locator, get_result_color_i32, get_outfit_data, get_result_scale_u16};

const MONSTER: [&str; 6] = [
    "JID_異形竜,MG_FireBreath",
    "JID_幻影竜,MG_FireBreath",
    "JID_異形狼",
    "JID_幻影狼",
    "JID_異形飛竜,MG_MirsmaBreath",
    "JID_幻影飛竜,MG_IceBreath"
];

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
        let result = AssetTable_Result::new();
        let conditions = AssetTable::s_condition_flags();
        let bond_face: Vec<_> = ResourceManager_2::get_s_files().iter().filter_map(|s| il2str(s.0).filter(|s| s.contains("Telop/LevelUp/FaceThumb/"))).collect();
        ["PID_リュール", "PID_M024_リュール", "GID_リュール"].iter().enumerate().for_each(|(i, lueur)|{
            let (hash, index) =
                if i < 2 {
                    let person = PersonData::get((*lueur).into());
                    (person.hash(), person.index())
                }
                else {
                    let god = GodData::get((*lueur).into());
                    (god.hash(), god.index())
                };
            ["男性", "女装"].iter().for_each(|gender|{
                result.clear();
                conditions.add_2(*lueur);
                conditions.add_2("MPID_Lueur");
                if i < 2 { conditions.add_2(PersonData::get((*lueur).into()).get_jid()); }
                conditions.add_2(*gender);
                result.commit(AssetTable_Modes::combat());
                result.replace(AssetTable_Modes::combat());
                if let Some(mut person) = PersonalDressData::from_asset_table(result, hashes, hash, i == 2){
                    result_hashes.insert(person.calc_hash());
                    if i == 1 { person.flags.set(PersonalDressDataFlags::Dark, true); }
                    person.flags.set(PersonalDressDataFlags::Lueur, true);
                    person.index = if i < 2 { index - 1 } else { index + 2000 } + person.flags.contains(PersonalDressDataFlags::Female) as i32;
                    person.mpid = "MPID_Lueur".to_string();
                    person.count = i as i32;
                    personal.push(person);
                }
            });
        });
        ["PID_青リュール_男性", "PID_青リュール_女性"].iter().enumerate().for_each(|v|{
            let person = PersonData::get((*v.1).into());
            let result = AssetTable_Result::get_from_pid(AssetTable_Modes::combat(), person.get_pid(), CharacterAppearance::conditions());
            if let Some(mut person) = PersonalDressData::from_asset_table(result, hashes, person.hash(), false){
                result_hashes.insert(person.calc_hash());
                person.flags.set(PersonalDressDataFlags::Female, v.0 == 1);
                person.flags.set(PersonalDressDataFlags::Alt, true);
                person.flags.set(PersonalDressDataFlags::Lueur, true);
                person.mpid = "MPID_Lueur".to_string();
                person.count = 3;
                personal.push(person);
            }
        });
        GodData::get_list().iter().filter(|g| !g.is_hero() && ( g.get_force_type().value == 0 || g.get_gid().to_rust_string().contains("GID_E006")))
            .for_each(|god|{
                let bond_face_path = format!("Telop/LevelUp/FaceThumb/{}", god.get_ascii_name());
                let has_bond_face = bond_face.contains(&bond_face_path);
                let dlc_dark = god.get_gid().to_rust_string().contains("GID_E006");
                let result = result.setup_2(AssetTable_Modes::combat(), god, false, CharacterAppearance::conditions());
                if let Some(mut person) = PersonalDressData::from_asset_table(result, hashes, god.hash(), true) {
                    person.flags.set(PersonalDressDataFlags::Dark, dlc_dark);
                    result_hashes.insert(person.calc_hash());
                    person.mpid = god.get_mid().to_rust_string();
                    person.index = if dlc_dark { 2500 } else { 2000 } + god.index();
                    person.count =
                        if let Some(count) = mpid_count.get_mut(&person.mpid) { *count += 1;*count }
                        else { mpid_count.insert(person.mpid.clone(), 0);0 };
                    person.flags.set(PersonalDressDataFlags::HasThumbnail, !FaceThumbnail::get_3(god).is_null());
                    person.flags.set(PersonalDressDataFlags::HasBondFace, has_bond_face);
                    personal.push(person)
                }
                if god.get_flag().m_value() & 32 == 0 && !dlc_dark {
                    result.clear();
                    let result = result.setup_2(AssetTable_Modes::combat(), god, true, CharacterAppearance::conditions());
                    if let Some(mut person) = PersonalDressData::from_asset_table(result, hashes, god.hash(), true) {
                        person.flags.set(PersonalDressDataFlags::Dark, true);
                        let result_hash = person.calc_hash();
                        if !result_hashes.contains(&result_hash) {
                            result_hashes.insert(result_hash);
                            person.flags.set(PersonalDressDataFlags::HasThumbnail, !FaceThumbnail::get_3(god).is_null());
                            person.flags.set(PersonalDressDataFlags::HasBondFace, has_bond_face);
                            person.mpid = god.get_mid().to_rust_string();
                            person.index = god.index() + 2500;
                            person.count =
                                if let Some(count) = mpid_count.get_mut(&person.mpid) { *count += 1;*count }
                                else { mpid_count.insert(person.mpid.clone(), 0);0 };
                            personal.push(person)
                        }
                    }
                }
            });

        PersonData::get_list()
            .iter()
            .filter(|p|{
                (p.get_belong().is_null() || p.get_pid().to_rust_string().contains("Boss")) &&
                    p.get_gender().value != 0 && il2str(p.get_name()).is_some_and(|v| !v.contains("Lueur"))
            })
            .for_each(|p|{
                let result = AssetTable_Result::get_from_pid(AssetTable_Modes::combat(), p.get_pid(), CharacterAppearance::conditions());
                let belong = p.get_belong();
                if let Some(mut person) = PersonalDressData::from_asset_table(result, hashes, p.hash(), false){
                    let hash = person.calc_hash();
                    if !result_hashes.contains(&hash) {
                        result_hashes.insert(hash);
                        person.index = p.index();
                        person.flags.set(PersonalDressDataFlags::GenericPerson, !p.get_name().to_rust_string().contains("Boss") && !belong.is_null());
                        person.flags.set(PersonalDressDataFlags::HasJob, !p.get_job().is_null());
                        person.flags.set(PersonalDressDataFlags::HasThumbnail, !FaceThumbnail::get_2(p).is_null());
                        let bond_face_path = format!("Telop/LevelUp/FaceThumb/{}", p.get_ascii_name());
                        person.flags.set(PersonalDressDataFlags::HasBondFace, bond_face.contains(&bond_face_path));
                        person.count =
                            if let Some(count) = mpid_count.get_mut(&person.mpid) { *count += 1;*count }
                            else { mpid_count.insert(person.mpid.clone(), 0);0 };
                        personal.push(person);
                    }
                }
            });
        personal.sort_by(|a, b| a.index.cmp(&b.index));
        println!("Appearance Count: {}", personal.len());
        let mut transform: Vec<JobTransformData> =
            MONSTER.iter().flat_map(|s|{
                if let Some((jid, magic)) = s.split_once(",") { JobTransformData::from_jid(jid, Some(magic)) }
                else { JobTransformData::from_jid(s, None) }
            }).collect();
        PersonData::get_list().iter().filter(|p| !p.get_job().is_null() && !p.get_aid().is_null())
            .for_each(|p|{
                let jhash = p.get_job().hash();
                if !transform.iter().any(|c| c.hash == jhash) {
                    if let Some(j) = JobTransformData::from_person(p) { transform.push(j); }
                }
            });
        let search_lists = AssetTable::s_search_lists();
        let job_conditions = JobData::get_list().iter().filter(|j| !job.iter().any(|x| x.hash == j.hash()))
            .flat_map(|j| get_condition_index(j.get_jid()).zip(Some(j.hash())))
            .collect::<Vec<(i32, i32)>>();
        let mode_2: Vec<_> =
            search_lists.get(2).iter()
                .filter(|x| !x.get_ride_dress_model().is_null() && !x.get_dress_model().is_null())
                .map(|x| {
                    (
                        il2str(x.get_dress_model()).filter(|c| c.contains("M_c") || c.contains("F_c") || c.contains("m_c") || c.contains("f_c")),
                        il2str(x.get_ride_dress_model()),
                        job_conditions.iter().filter(|(_, c)| has_condition_index(x, *c)).map(|v| v.0).collect::<Vec<i32>>()
                    )
                }).collect();
        let mode_1: Vec<_> =
            search_lists.get(1).iter()
                .filter(|x| !x.get_ride_model().is_null() && !x.get_body_model().is_null())
                .map(|x| {
                    (
                        il2str(x.get_body_model()).filter(|c| c.contains("M_c") || c.contains("F_c") || c.contains("m_c") || c.contains("f_c")),
                        il2str(x.get_ride_model()),
                        job_conditions.iter().filter(|(_, c)| has_condition_index(x, *c)).map(|v| v.0).collect::<Vec<i32>>()
                    )
                }).collect();

        job_conditions.iter().for_each(|&(hash, condition)|{
            let mode_1m = mode_1.iter()
                .find(|x| x.2.contains(&hash) && (x.0.as_ref().is_some_and(|c| c.contains("M_c") || c.contains("m_c")))).and_then(|v| v.0.clone());

            let mode_1f = mode_1.iter()
                .find(|x| x.2.contains(&hash) && (x.0.as_ref().is_some_and(|c| c.contains("F_c") || c.contains("f_c")))).and_then(|v| v.0.clone());

            let mode_2m = mode_2.iter()
                .find(|x| x.2.contains(&hash) && (x.0.as_ref().is_some_and(|c| c.contains("M_c") || c.contains("m_c")))).and_then(|v| v.0.clone());

            let mode_2f = mode_2.iter()
                .find(|x| x.2.contains(&hash) && (x.0.as_ref().is_some_and(|c| c.contains("F_c") || c.contains("f_c")))).and_then(|v| v.0.clone());

            let mode_1r = mode_1.iter().find(|x| x.2.contains(&hash) && x.1.is_some()).and_then(|v| v.1.clone());
            let mode_2r = mode_2.iter().find(|x| x.2.contains(&hash) && x.1.is_some()).and_then(|v| v.1.clone());
            let mount = mode_2r.as_ref().map(|s| Mount::determine_mount(s.as_str())).unwrap_or(Mount::None);
            if let Some((_, gender)) = mode_2m.as_ref().and_then(|s| Mount::determine_gender(s.as_str())) {
                job.push(
                    JobDressData {
                        hash, mount, gender, hair_color: 0,
                        dress_model: mode_2m.unwrap(), ride_dress: mode_2r.clone(),
                        ride_body: mode_1r.clone(), body_model: mode_1m,
                    }
                );
            }
            if let Some((_, gender)) = mode_2f.as_ref().and_then(|s| Mount::determine_gender(s.as_str())) {
                job.push(
                    JobDressData {
                        hash, mount, gender, hair_color: 0,
                        dress_model: mode_2f.unwrap(), body_model: mode_1f,
                        ride_dress: mode_2r, ride_body: mode_1r,
                    });
            }
        });
        Self { job, engaged, personal, transform}
    }
    pub fn get_engaged_dress(&self, asset: Il2CppString) -> Option<&EngagedDressData> {
        if asset.is_null() { None }
        else {
            let mut str = asset.to_string();
            if str.starts_with("EID_") { str = str.trim_start_matches("EID_").to_string(); }
            self.engaged.iter().find(|x| x.asset_id == str)
        }
    }
    pub fn get_job_dress(&self, job: JobData, gender: engage::app::Gender) -> Option<&JobDressData> {
        self.job.iter().find(|x| x.is_match(gender, job))
    }
    pub fn get_personal_dress(&self, unit: engage::app::Unit) -> Option<&PersonalDressData> {
        if unit.is_null() || unit.get_person().is_null() || unit.get_person().get_job().is_null() { None }
        else if unit.m_edit().is_enable() {
            self.get_personal_dress_by_person(unit.get_person(), unit.m_edit().m_gender().value ==2)
        }
        else {
            let person = unit.get_person();
            let is_female = unit.get_dress_gender() == engage::app::Gender::female();
            if person.get_flag().m_value() & 512 == 0 { self.get_personal_dress_by_person(person, is_female) }
            else { None }
        }
    }
    pub fn get_personal_dress_by_person(&self, person: PersonData, female: bool) -> Option<&PersonalDressData> {
        let is_lueur = person.index() == 1 || person.get_flag().m_value() & 128 != 0;
        self.personal.iter().find(|x|
            x.hash == person.hash() &&
                (x.flags.contains(PersonalDressDataFlags::Lueur) == is_lueur) && (x.flags.contains(PersonalDressDataFlags::Female) == female)
        ).or_else(||
            il2str(person.get_name())
                .and_then(|name|self.personal.iter().find(|x| x.mpid == name && x.flags.contains(PersonalDressDataFlags::Female) == female))
        )
    }
    pub fn get_personal_dress_by_name(&self, name: &str, female: bool) -> Option<&PersonalDressData> {
        self.personal.iter().find(|x| x.flags.contains(PersonalDressDataFlags::Female) == female && x.mpid == name)
    }
}
#[derive(Default, Clone)]
pub struct PersonalDressData {
    pub mpid: String,
    pub data_hash: i32,
    pub flags: PersonalDressDataFlags,
    pub ubody: i32,
    pub uhair: i32,
    pub uhead: i32,
    pub ohair: i32,
    pub color: [i32; 16],
    pub scale: [u16; 19],
    pub mount: Option<(Mount, i32)>,
    pub acc: [i32; 5],
    pub aoc: [i32; 4],
    pub hash: i32,
    pub index: i32,
    pub count: i32,
    pub voice: i32,
    pub engage_hair: i32,
}
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct PersonalDressDataFlags: i32 {
        const Female = 1;
        const FromPerson = 1 << 1;
        const FromGod = 1 << 2;
        const Dark = 1 << 3;
        const GenericPerson = 1 << 4;
        const Morph = 1 << 5;
        const Alt = 1 << 6;
        const HasJob = 1 << 7;
        const Lueur = 1 << 8;
        const HasThumbnail = 1 << 9;
        const HasBondFace = 1 << 10;
    }
}
impl Default for PersonalDressDataFlags {
    fn default() -> Self { PersonalDressDataFlags::empty() }
}
impl PersonalDressDataFlags {
    pub fn valid_for_playable(&self) -> bool {
        !self.contains(PersonalDressDataFlags::GenericPerson) && self.contains(PersonalDressDataFlags::HasJob)
    }
}
impl PersonalDressData {
    pub fn calc_hash(&self) -> i64 {
        let mut hash = (self.uhead as i64) + ((self.uhair as i64) << 2) + ((self.uhead as i64) << 4);
        for x in 4..8 { hash += (self.color[x] << x) as i64; }
        hash
    }
    pub fn process_map_result(&mut self, result: AssetTable_Result, hash_list: &OutfitHashes) {
        let head_model = result.get_head_model();
        if !head_model.is_null() {
            let hash = head_model.get_hash_code();
            if hash_list.o_hair.contains_key(&hash) { self.ohair = hash; }
        }
        for x in 0..8 { self.color[x+8] = get_result_color_i32(result, x); }
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
        if head.contains("c7") { self.flags.set(PersonalDressDataFlags::Morph, true); }
        let hair = result.get_hair_model();
        if !hair.is_null() {
            let h = hair.get_hash_code();
            let hair = hair.to_rust_string();
            if hair.contains("801") || hair.contains("851") { generic_count += 1; }
            if hash_list.hair.contains_key(&h) { uhair = h; }
        }
        if let Some(model) = try_find_accessory_model(result, "_Hair") {
            let h = Il2CppString::from(model).get_hash_code();
            if hash_list.hair.contains_key(&h) { uhair = h }
        }
        if generic_count >= 2 || ubody == 0 || uhead == 0 { return false; }
        self.ubody = ubody;
        self.uhead = uhead;
        self.uhair = uhair;
        self.flags.set(PersonalDressDataFlags::Female, hash_list.female_u.contains(&ubody));
        for i in 0..5 {
            if let Some(model) = try_get_model_at_locator(result, ACC_LOC[i]){
                let h = Il2CppString::from(model).get_hash_code();
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
        result.set_head_model(Il2CppString::null());
        for x in 0..8 { set_color_by_i32(result, x, 0); }
        for x in 0..19 { crate::set_result_scale_u16(result, x, 100); }
        result.commit(AssetTable_Modes::onmap());
        result.replace(AssetTable_Modes::onmap());
        self.process_map_result(result, hash_list);
        true
    }
    pub fn from_asset_table(result: AssetTable_Result, hash_list: &OutfitHashes, hash: i32, emblem: bool) -> Option<PersonalDressData> {
        let mut new = PersonalDressData::default();
        new.hash = hash;
        new.flags.set(PersonalDressDataFlags::FromGod, emblem);
        new.flags.set(PersonalDressDataFlags::FromPerson, !emblem);
        new.mpid =
            if emblem { GodData::try_get_from_hash(hash).get_mid() } else { PersonData::try_get_from_hash(hash).get_name() }.to_rust_string();
        if !new.process_from_asset_table(result, &hash_list) { None } else { Some(new) }
    }
    pub fn get_menu_name(&self) -> Il2CppString {
        if self.count == 0 { Mess::get(self.mpid.as_str()) }
        else { format!("{} {}", Mess::get(self.mpid.as_str()), self.count + 1).into() }
    }
    pub fn apply(&self, result: AssetTable_Result, mode: i32, promoted: bool, mount: Option<Mount>, outfit_hashes: &OutfitHashes) {
        if mode == 2 {
            if let Some(ubody) = outfit_hashes.body.get(&self.ubody) {
                result.set_dress_model(ubody.as_str());
            }
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
            if il2str(result.get_head_model()).is_none_or(|v| v.contains("null")){
                if let Some(head) = outfit_hashes.get_ohair(self.uhead) { result.set_head_model(head); }
            }
        }
    }
    pub fn get_thumbnail_key(&self) -> Option<Sprite> {
        if self.flags.contains(PersonalDressDataFlags::Lueur) && !self.flags.contains(PersonalDressDataFlags::Dark){
            let mut key = "Lueur".to_string();
            if self.flags.contains(PersonalDressDataFlags::Female) { key.push('W'); }
            if self.flags.contains(PersonalDressDataFlags::Alt) || self.flags.contains(PersonalDressDataFlags::FromGod) {
                key.push_str("_God");
            }
            let sprite = FaceThumbnail::s_face_thumb().try_get(key);
            if sprite.is_null() { None } else { Some(sprite) }
        }
        else if self.flags.contains(PersonalDressDataFlags::HasThumbnail) {
            if self.flags.contains(PersonalDressDataFlags::FromGod){
                let sprite = FaceThumbnail::get_3(GodData::try_get_from_hash(self.hash));
                if sprite.is_null() { None } else { Some(sprite) }
            }
            else if self.flags.contains(PersonalDressDataFlags::FromPerson){
                let sprite = FaceThumbnail::get_2(PersonData::try_get_from_hash(self.hash));
                if sprite.is_null() { None } else { Some(sprite) }
            }
            else {
                let sprite = FaceThumbnail::s_face_thumb().try_get("Phantom");
                if sprite.is_null() { None } else { Some(sprite) }
            }
        }
        else {
            let sprite =  FaceThumbnail::s_face_thumb().try_get("Phantom");
            if sprite.is_null() { None } else { Some(sprite) }
        }
    }
    pub fn get_bond_face_path(&self) -> Option<String> {
        if self.flags.contains(PersonalDressDataFlags::Lueur) && !self.flags.contains(PersonalDressDataFlags::Dark){
            let emblem = self.flags.contains(PersonalDressDataFlags::Alt) || self.flags.contains(PersonalDressDataFlags::FromGod);
            let female = self.flags.contains(PersonalDressDataFlags::Female);
            Some(format!("Telop/LevelUp/FaceThumb/{}Lueur{}", if emblem { "God" } else { "" }, if female { "W" } else { "" }))
        }
        else if self.flags.contains(PersonalDressDataFlags::HasBondFace) {
            if self.flags.contains(PersonalDressDataFlags::FromGod) {
                Some(TelopManager::get_bond_level_face_path_3(GodData::try_get_from_hash(self.hash)).to_rust_string())
            }
            else if self.flags.contains(PersonalDressDataFlags::FromPerson) {
                let person = PersonData::try_get_from_hash(self.hash);
                let mut key = "Telop/LevelUp/FaceThumb/".to_string();
                key += person.get_ascii_name().to_rust_string().as_str();
                Some(key)
            }
            else { None }
        }
        else { None }
    }
    pub fn get_unit_icon(&self, dark: bool) -> Option<String> {
        if self.flags.contains(PersonalDressDataFlags::Lueur) {
            let female = self.flags.contains(PersonalDressDataFlags::Female);
            if self.flags.contains(PersonalDressDataFlags::Dark) || dark { Some(if female { "052Lueur_719" } else { "002Lueur_718"}.to_string() + "ShadowLord_NoWeapon") }
            else if self.flags.contains(PersonalDressDataFlags::Alt) || self.flags.contains(PersonalDressDataFlags::FromGod){
                Some(if female { "05" } else { "00" }.to_string() + "1LueurE_001Lueur_NoWeapon")
            }
            else { Some(if female { "052Lueur_601"} else { "001Lueur_600"}.to_string() + "DragonLord_NoWeapon") }
        }
        else if self.flags.contains(PersonalDressDataFlags::FromPerson) && self.flags.contains(PersonalDressDataFlags::HasJob) {
            let person = PersonData::try_get_from_hash(self.hash);
            let gender = person.get_gender();
            let unit_icon = person.get_unit_icon_id();
            let job = person.get_job();
            let job_icon = job.get_unit_icon_id(gender.value == 2);
            let weapon_icon = job.get_unit_icon_weapon_id();
            let key = format!("{}_{}_{}", unit_icon, job_icon, weapon_icon);
            println!("Looking for: {}", key);
            Some(key)
        }
        else if self.flags.contains(PersonalDressDataFlags::FromGod){
            let god = GodData::try_get_from_hash(self.hash);
            let icon = god.get_unit_icon_id();
            if self.flags.contains(PersonalDressDataFlags::Dark) || dark {
                let key = format!("997Darkness_{}_NoWeapon", icon);
                println!("Looking for: {}", key);
                if engage::app::GameIcon::tyr_get_unit_icon_index(key.as_str()).is_null() { Some("997Darkness_711Shadow_NoWeapon".to_string()) }
                else { Some(key) }
            }
            else { Some(format!("{}_{}_NoWeapon", icon, icon)) }
        }
        else { None }
    }
    pub fn get_name(&self) -> Il2CppString { Mess::get(self.mpid.as_str()) }
    pub fn apply_appearance(&self, result: AssetTable_Result, mode: i32, promoted: bool, mount: Option<Mount>, outfit_hashes: &OutfitHashes, remove_empty_acc: bool) {
        self.apply(result, mode, promoted, mount, outfit_hashes);
        if mode == 2 {
            if let Some(uhead) = outfit_hashes.head.get(&self.uhead) { result.set_head_model(uhead.as_str()); }
            if let Some(uhair) = outfit_hashes.hair.get(&self.uhair) { apply_result_hair(uhair, result); }
            for x in 0..4 {
                if let Some(acc) = outfit_hashes.acc.get(&self.acc[x]){
                    result.commit_8(new_asset_table_accessory(acc.as_str(), ACC_LOC[x]));
                }
                else if remove_empty_acc && x != 4 { result.commit_8(new_asset_table_accessory("null", ACC_LOC[x])); }
            }
            if try_get_model_at_locator(result, ACC_LOC[4]).is_none() { result.commit_8(new_asset_table_accessory("null", ACC_LOC[4])); }
            for x in 0..16 {
                let v = self.scale[x];
                if v > 0 { crate::set_result_scale_u16(result, x, v); }
            }
            for x in 0..8 {
                let color = self.color[x];
                if color > 0 { set_color_by_i32(result, x, self.color[x]); }
            }
        }
        else {
            if let Some(ohair) = outfit_hashes.o_hair.get(&self.ohair) { result.set_head_model(ohair.as_str()); }
            else if let Some(ohair) = outfit_hashes.get_ohair(self.uhair).or_else(|| outfit_hashes.get_ohair(self.uhead)){ result.set_head_model(ohair); }
            else if self.uhair != 0 { result.set_head_model( if self.flags.contains(PersonalDressDataFlags::Female) { "oHair_h850" } else { "oHair_h800" }); }
            for x in 0..4 {
                if let Some(acc) = outfit_hashes.get_oacc(self.acc[x]){
                    result.commit_8(new_asset_table_accessory(acc.to_rust_string().as_str(), ACC_LOC[x]));
                }
            }
            for x in 0..8 {
                let color = self.color[x+8];
                if color > 0 { set_color_by_i32(result, x, self.color[x+8]); }
            }
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
        self.hash == person_hash || il2str(person.get_name()).is_some_and(|name| name.to_string() == self.mpid)
    }
    pub fn get_male_indexes(for_playable: bool, filter: impl Fn(&PersonalDressData) -> bool) -> Vec<usize> {
        let db = &get_outfit_data().dress.personal;
        db.iter().enumerate()
            .filter(|v| filter(v.1) && !v.1.flags.contains(PersonalDressDataFlags::Female) && ((for_playable && v.1.flags.valid_for_playable()) || !for_playable))
            .map(|v| v.0)
            .collect()
    }
    pub fn get_female_indexes(for_playable: bool, filter: impl Fn(&PersonalDressData) -> bool ) -> Vec<usize> {
        let db = &get_outfit_data().dress.personal;
        db.iter().enumerate()
            .filter(|v| filter(v.1) && v.1.flags.contains(PersonalDressDataFlags::Female) && ((for_playable && v.1.flags.valid_for_playable()) || !for_playable))
            .map(|v| v.0)
            .collect()
    }
}
pub struct JobTransformData {
    pub hash: i32,
    pub is_transform: bool,
    pub mode_2_conditions: Vec<String>,
    pub magic: Option<String>,

}
impl JobTransformData {
    pub fn check_result(result: AssetTable_Result) -> bool {
        il2str(result.get_dress_model()).is_some_and(|v| v.contains("AT_c")) && !result.get_body_model().is_null() && !result.get_body_anim().is_null()
    }
    pub fn from_jid(jid: &str, magic: Option<&'static str>) -> Option<Self>{
        let job_data = JobData::get(jid.into());
        if !job_data.is_null() {
            Some(
                Self{
                    hash: job_data.hash(),
                    is_transform: false,
                    mode_2_conditions: vec![jid.to_string()],
                    magic: magic.map(|m| m.to_string()),
                }
            )
        }
        else { None }
    }
    pub fn from_person(person: PersonData) -> Option<JobTransformData> {
        if person.get_job().is_null() { return None; }
        let result = AssetTable_Result::new();
        let flags = AssetTable::s_condition_flags();
        result.clear();
        let mut conditions = vec![];
        let job = person.get_job();
        let jid = person.get_job().get_jid();
        let mode = AssetTable_Modes::combat();
        if let Some(aid) = il2str(person.get_aid()) {
            flags.add_2(aid.as_str());
            conditions.push(aid);
            result.commit(mode);
            result.replace(mode);
            if Self::check_result(result) {
                println!("Transform for {}", Mess::get(job.get_name()));
                return Some(
                    Self {
                        hash: job.hash(),
                        is_transform: true,
                        mode_2_conditions: conditions,
                        magic: None,
                    }
                );
            }
        }
        else {
            flags.add_2("Transformed");
            flags.add_2(jid);
            result.commit(mode);
            result.replace(mode);
            if Self::check_result(result) {
                println!("Transform for {}", Mess::get(job.get_name()));
                conditions.push(jid.to_rust_string());
                conditions.push("Transformed".to_string());
                return Some(
                    Self {
                        hash: job.hash(),
                        is_transform: true,
                        mode_2_conditions: conditions,
                        magic: None,
                    }
                );
            }
        }
        None
    }
    pub fn get_result(&self, mode: i32, unit: engage::app::Unit) -> AssetTable_Result{
        let result = AssetTable_Result::new();
        result.clear();
        let conditions = AssetTable::s_condition_flags();
        conditions.add_2(unit.get_person().get_name());
        conditions.add_2(unit.get_pid());
        self.mode_2_conditions.iter().for_each(|key|{ conditions.add_2(key.as_str()); });
        result.commit(AssetTable_Modes{value: mode});
        result.replace(AssetTable_Modes{ value: mode });
        if let Some(magic) = self.magic.as_ref() { result.set_magic(magic.as_str()); }
        result
    }
    pub fn set_result(&self, mode: i32, unit: engage::app::Unit, result: AssetTable_Result) {
        result.clear();
        let conditions = AssetTable::s_condition_flags();
        conditions.add_2(unit.get_person().get_name());
        conditions.add_2(unit.get_pid());
        if mode == 1 && self.is_transform { conditions.add_2("竜化"); }
        self.mode_2_conditions.iter().for_each(|key|{ conditions.add_2(key.as_str()); });
        result.commit(AssetTable_Modes{value: mode});
        result.replace(AssetTable_Modes{ value: mode });
        if let Some(magic) = self.magic.as_ref() { result.set_magic(magic.as_str()); }
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