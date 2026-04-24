use std::collections::HashSet;
use std::fs::{read_to_string, DirEntry};
use engage::gamedata::assettable::AssetTableResult;
use engage::gamedata::{Gamedata, GodData, PersonData};
use engage::unit::Gender;
use engage::gameuserdata::GameUserData;
use engage::mess::Mess;
use engage::stream::Stream;
use unity::prelude::*;
use crate::assets::new_asset_table_accessory;
use crate::{apply_hair, get_outfit_data, AssetColor, AssetType, Mount, OutfitData, PersonalDressData, UnitAssetMenuData, OUTFIT_DATA};
use crate::AssetType::Acc;
const PLAYABLE_HASH: [i32; 41] = [
    276380359,152765422,1875144918,1654010808,-594922007,7981978,1201591043,-59016776,
    1808009585,1348996286,1172357650,-1768838071,-204100902,-1916470567,473157409,1486827994,
    -1837275069,-1738982548,-1130667140,621398521,-560458477,1533710081,459790438,49112263,
    929747596,-1947925903,497864924,-266109647,469588104,2072087003,1276288756,917195929,
    356559395,2008756543,1975241240,-2049428753,-1888964692,1696364213,1740718976,566028171,754290911
];

const SCALE_NAME: [&str; 16] = [
    "All", "Head", "Neck", "Torso", "Shoulder", "Arms", "Hands", "Legs", "Feet", "V_Bust", "V_Abdomen", "V_Torso",
    "V_BaseArms", "V_BaseLegs", "V_Arms", "V_Legs"
];
const COLORS: [&str; 8] = ["HairColor", "HairGrad", "Skin", "Toon", "Mask100", "Mask75", "Mask50", "Mask25"];

const VAR_NAMES: [&str; 27] = [
    "Body", "Head", "Hair", "HeadAcc1", "HeadAcc2", "SpineAcc", "Trans", "Shield",
    "InfoAnim", "TalkAnim", "DemoAnim", "HubAnim", "InfoAnimM", "TalkAnimM",
    "DemoAnimM", "HubAnimM", "InfoAnimF", "TalkAnimF", "DemoAnimF", "HubAnimF",
    "Horse", "Wolf", "Wyvern", "Pegasus", "Griffin", "Voice", "Rig",
];
pub const ACC_LOC: [&str; 5] = ["c_head_loc", "c_head2_loc", "c_spine2_jnt", "c_trans", "l_shld_loc", ];

const DEFAULT_AOC: [&str; 8] = [
    "AOC_Info_c000", "AOC_Talk_c000", "AOC_Demo_Hum0M", "AOC_Hub_Hum0M",
    "AOC_Info_c050", "AOC_Talk_c050", "AOC_Demo_Hum0F", "AOC_Hub_Hum0F"
];

#[derive(Clone)]
pub struct UnitAssetData {
    pub person: i32,
    pub flag: i32,
    pub set_profile: [i32; 5], // Battle Engaged Hub, Exploration, Cutscene
    pub profile: Vec<PlayerOutfitData>,
}

impl UnitAssetData {
    pub fn version() -> i32 { 8 }
    pub fn new_hash(hash: i32, random_app: bool) -> Self {
        let (profile, flag) =
        if GodData::try_get_hash(hash).is_some() { (vec![PlayerOutfitData::new_with_flag(0); 3], 1) }
        else { (vec![PlayerOutfitData::new_with_flag(0); 5], if random_app { 8 } else { 0 }) };
        Self { person: hash, profile, set_profile: [0, 1, 2, 0, 0], flag, }
    }
    pub fn serialize(&self, stream: &mut Stream){
        let _ = stream.write_int(self.person).unwrap();
        let _ = stream.write_int(self.flag).unwrap();
        let _ = stream.write_int(self.profile.len() as i32).unwrap();
        self.set_profile.iter().for_each(|m|{ stream.write_int(*m).unwrap(); });
        self.profile.iter().for_each(|m|{ m.serialize(stream); });
    }
    pub fn deserialize(stream: &mut Stream, version: i32) -> Self {
        let mut set_profile: [i32; 5] = [0, 1, 2, 0, 0];
        let person = stream.read_int().unwrap_or(0);
        let flag = stream.read_int().unwrap_or(0);
        let person_flag = flag;
        let count = stream.read_int().unwrap_or(0);
        for x in 0..5 { set_profile[x] = stream.read_int().unwrap_or(0); }
        let mut profile = vec![];
        for _ in 0..count { profile.push(PlayerOutfitData::deserialize(stream, version)); }
        Self { set_profile, person, profile, flag: person_flag }
    }
    pub fn profile_index(&self, engaged_dark: bool) -> i32 {
        let index = if GameUserData::get_sequence() == 4 { 2 } else if engaged_dark { 1 } else { 0 };
        self.set_profile[index as usize]
    }
    pub fn get_active_flag(&self, engaged: bool) -> i32 {
        let i = self.profile_index(engaged);
        self.profile.get(i as usize).map(|v| v.flag).unwrap_or(0)
    }
    pub fn set_result(&self, result: &mut AssetTableResult, mode: i32, engaged: bool, stun: bool) {
        let index = self.profile_index(engaged);
        if let Some(profile) = self.profile.get(index as usize) { profile.set_result(result, mode, engaged, stun); }
    }
    pub fn set_color(&self, result: &mut AssetTableResult, engaged: bool) {
        let index = self.profile_index(engaged);
        if let Some(profile) = self.profile.get(index as usize) { profile.set_color(result); }
    }
}
#[derive(Clone, PartialEq)]
pub struct PlayerOutfitData {
    pub flag: i32,
    pub ubody: i32,
    pub uhead: i32,
    pub uhair: i32,
    pub aoc: [i32; 4],  //  Info, Talk, Demo, Hub  Male
    pub colors: [AssetColor; 8],
    pub scale: [u16; 18],
    pub break_body: i32,
    pub acc: [i32; 5],
    pub mount: [i32; 5],
    pub voice: i32,
    pub rig: i32,
    pub aoc_alt: [i32; 4],  // Info, Talk, Demo, Hub Female
}
impl PlayerOutfitData {
    pub const fn new() -> Self {
        Self {
            flag: 0, uhair: 0, uhead: 0, aoc: [0; 4], scale: [0; 18], acc: [0; 5], voice: 0, mount: [0; 5],
            break_body: 0,
            ubody: 0, colors: [AssetColor::new(); 8], rig: 0, aoc_alt: [0; 4],
        }
    }
    pub fn from_appearance(appearance: &PersonalDressData) -> Self {
        let mut new = Self::new();
        new.ubody = appearance.ubody;
        new.uhead = appearance.uhead;
        new.uhair = appearance.uhair;
        new.voice = appearance.voice;
        if let Some(m) = appearance.mount.as_ref() {
            if (m.0 as i32) > 0 { 
                new.mount[m.0 as usize - 1] = m.1;
            }
        }
        for x in 0..4 { 
            new.aoc[x] = appearance.aoc[x]; 
            new.aoc_alt[x] = appearance.aoc[x];
        }
        for x in 0..5 { new.acc[x] = appearance.acc[x]; }
        for x in 0..16 { new.scale[x] = appearance.scale[x]; }
        for x in 0..8 { new.colors[x].set_by_i32(appearance.color[x]); }
        new
    }
    pub fn new_with_flag(flag: i32) -> Self {
        Self {
            flag, ubody: 0, uhead: 0, aoc: [0; 4], scale: [0; 18],
            break_body: 0,
            uhair: 0,
            mount: [0; 5],
            colors: [AssetColor::new(); 8], acc: [0; 5],
            voice: 0, rig: 0, aoc_alt: [0; 4],
        }
    }
    pub fn set_from_preset(&mut self, data: &PersonalDressData) {
        let photo = UnitAssetMenuData::is_photo_graph();
        self.uhair = data.uhair;
        self.ubody = data.ubody;
        self.uhead = data.uhead;
        if !photo {
            let db = get_outfit_data();
            for x in 0..4 {
                match db.get_aoc_gender_hash(x as i32, data.aoc[x]) {
                    Some(Gender::Male) => { self.aoc[x] = data.aoc[x]; }
                    Some(Gender::Female) => { self.aoc_alt[x] = data.aoc[x]; }
                    _ => {}
                }
            }
        }
        for x in 0..16 { self.scale[x] = data.scale[x] }
        for x in 0..5 {
            if data.acc[x] == 0 {
                self.acc[x] = Il2CppString::new("null").get_hash_code();
            } else if !(x == 4 && photo) { self.acc[x] = data.acc[x] }
        }
        for x in 0..8 { self.colors[x].set_by_i32(data.color[x]); }
    }
    pub fn is_empty(&self, _gender: Option<Gender>) -> bool {
        let db = get_outfit_data();
        let not_empty =
        self.colors.iter().any(|v| v.has_color()) ||
            self.scale.iter().any(|v| *v > 0 && * v < 1000 ) ||
            db.try_get_asset(AssetType::Head, self.uhead).is_some() ||
            db.try_get_asset(AssetType::Hair, self.uhair).is_some() ||
            db.try_get_asset(AssetType::Rig, self.rig).is_some() ||
            db.try_get_asset(AssetType::Voice, self.voice).is_some() ||
            self.mount.iter().enumerate().any(|(i, m)| db.try_get_asset(AssetType::Mount(i as u8), *m).is_some()) ||
            self.acc.iter().any(|a| db.try_get_asset(Acc(0), *a).is_some());
        if not_empty { false }
        /*
        else if let Some(g) = gender.filter(|g| *g == Gender::Male || *g == Gender::Female){
            if db.get_dress_gender_hash(self.ubody) == Some(g) { false }
            else if g == Gender::Male { !self.aoc.iter().any(|a| db.try_get_asset(AssetType::AOC(0), *a).is_some()) }
            else { !self.aoc_alt.iter().any(|a| db.try_get_asset(AssetType::AOC(0), *a).is_some()) }
        }
        */
        else { true }
    }
    pub fn deserialize(stream: &mut Stream, version: i32) -> Self {
        let flag = stream.read_int().unwrap_or(0);
        let ubody = stream.read_int().unwrap_or(0);
        let uhead = stream.read_int().unwrap_or(0);
        let uhair = stream.read_int().unwrap_or(0);
        let rig = if version >= 7 { stream.read_int().unwrap_or(0) } else { 0 };
        let mut aoc = [0; 4];
        aoc.iter_mut().for_each(|x| *x = stream.read_int().unwrap_or(0));
        let mut colors = [AssetColor::new(); 8];
        let mut mount = [0; 5];
        for x in 0..8 { colors[x] = AssetColor::from_stream(stream); }
        let mut scale: [u16; 18] = [0; 18];
        for x in 0..18 {
            let mut v = stream.read_u16().unwrap_or(0);
            if v > 1000 { v = 0;}
            scale[x] = v;
        }
        let break_body = stream.read_int().unwrap_or(0);
        let mut acc: [i32; 5] = [0; 5];
        for x in 0..5 { acc[x] = stream.read_int().unwrap_or(0); }
        mount.iter_mut().for_each(|m|{ *m = stream.read_int().unwrap_or(0); });
        let voice = stream.read_int().unwrap_or(0);
        let mut aoc_alt = [0; 4];
        if version >= 8 {
            for x in 0..4 { aoc_alt[x] = stream.read_int().unwrap_or(0); }
        }
        else {
            let db = get_outfit_data();
            for x in 0..4 {
                if db.get_aoc_gender_hash(x, aoc[x as usize]).is_some_and(|x| x == Gender::Female){
                    aoc_alt[x as usize] = aoc[x as usize];
                    aoc[x as usize] = 0;
                }
            }
        }
        Self { flag, ubody, uhead, uhair, aoc, colors, break_body, scale, acc, voice, mount, rig, aoc_alt }
    }
    pub fn serialize(&self, stream: &mut Stream) -> usize {
        let mut bytes = 0;
        bytes += stream.write_int(self.flag).unwrap_or(0);
        bytes += stream.write_int(self.ubody).unwrap_or(0);
        bytes += stream.write_int(self.uhead).unwrap_or(0);
        bytes += stream.write_int(self.uhair).unwrap_or(0);
        bytes += stream.write_int(self.rig).unwrap_or(0);
        self.aoc.iter().for_each(|a|{bytes += stream.write_int(*a).unwrap(); });
        self.colors.iter().for_each(|c|{ bytes += c.serialize(stream); });
        self.scale.iter().for_each(|s|{ bytes += stream.write_u16(*s).unwrap(); });
        stream.write_int(self.break_body).unwrap_or(0);
        self.acc.iter().for_each(|a| { bytes += stream.write_int(*a).unwrap(); });
        self.mount.iter().for_each(|m|{ bytes += stream.write_int(*m).unwrap(); });
        bytes += stream.write_int(self.voice).unwrap_or(0);
        self.aoc_alt.iter().for_each(|a|{bytes += stream.write_int(*a).unwrap(); });
        bytes
    }
    pub fn set_color(&self, result: &mut AssetTableResult) {
        if self.flag & 1 != 0 { for i in 0..8 { self.colors[i].set_result_color(result, i); } }
    }
    pub fn set_result(&self, result: &mut AssetTableResult, mode: i32, engaged: bool, stun: bool) {
        let sequence = GameUserData::get_sequence();
        let db = get_outfit_data();
        self.set_color(result);
        if sequence != 4 {
            if let Some(voice) = db.hashes.voice.get(&self.voice){ result.sound.voice = Some(voice.into()); }
        }
        if mode == 2 {
            let original_dress_gender = db.get_dress_gender(result.dress_model);
            if let Some(rig) = db.try_get_asset(AssetType::Rig, self.rig) { result.body_model = rig.into(); }
            if let Some(head) = db.try_get_asset(AssetType::Head, self.uhead) { result.head_model = head.into(); }
            if !self.colors[2].has_color() || self.flag & 1 == 0 {
                let head_hash = result.head_model.get_hash_code();
                if let Some(color) = db.list.skin.get(&head_hash) {
                    color.set_result_color(result, 2);
                }
            }
            if let Some(hair) = db.try_get_asset(AssetType::Hair, self.uhair) { apply_hair(hair, result); }
            if !engaged || (engaged && self.flag & 2 != 0) || (stun && self.flag & 32 != 0) {
                let allow_cross_dress = self.flag & 128 != 0;
                let b = if self.flag & 32 != 0 && stun { self.break_body } else { self.ubody };
                if let Some(body) = db.try_get_asset(AssetType::Body, b)
                    .or_else(|| db.try_get_asset(AssetType::Body, self.ubody))
                {
                    let new_dress_gender = db.get_dress_gender(body.into());
                    if original_dress_gender == new_dress_gender { result.dress_model = body.into(); }
                    else if allow_cross_dress { result.dress_model = body.into(); }
                }
                for x in 0..5 {
                    if let Some(head) = db.try_get_asset(AssetType::Acc(x as u8), self.acc[x]) {
                        if head.contains("Msc0AT") { result.left_hand = head.into(); } else {
                            let accessory = new_asset_table_accessory(head.to_string().as_str(), ACC_LOC[x]);
                            result.commit_accessory(&accessory);
                        }
                    }
                }
            }
            if self.flag & 64 != 0 {
                for x in 0..16 {
                    if self.scale[x] > 0 && self.scale[x] <= 1000 { result.scale_stuff[x] = (self.scale[x] as f32) / 100.0; }
                }
            }
            if let Some(ride_dress_model) = result.ride_dress_model {
                let current_mount = Mount::from(ride_dress_model.to_string().as_str());
                let mount_index = i32::from(current_mount) - 1;
                if mount_index >= 0 && mount_index < 5 {
                    let selection = self.mount[mount_index as usize];
                    if let Some(ride) = db.hashes.mounts.get(&selection) { result.ride_dress_model = Some(ride.into()); }
                }
            }
            let dress_gender = db.get_dress_gender(result.dress_model);

            let aoc_default_offset = if dress_gender == Gender::Male { 0 } else { 4 } as usize;
            for x in 0..4 {
                if let Some(anim) = result.get_anim(x as i32) {
                    let hash = if dress_gender == Gender::Male { self.aoc[x] } else { self.aoc_alt[x] };
                    if let Some(aoc) = db.try_get_asset(AssetType::AOC(x as u8), hash){
                        *anim = aoc.into();
                    }
                    else {
                        let anim_gender = db.get_aoc_gender(x as i32, anim);
                        if anim_gender != dress_gender && anim_gender != Gender::None {
                            *anim = DEFAULT_AOC[aoc_default_offset + x].into();
                        }
                    }
                }
            }
            result.replace(2);
        }
        else {
            // if let Some(skin) = db.list.skin.get(&self.uhead) { ColorPreset::set_color(&mut result.unity_colors[2], *skin); }
            if !engaged || (engaged && self.flag & 2 != 0) {
                let original_dress_gender = db.get_dress_gender(result.body_model);
                let b = if self.flag & 32 != 0 && stun { self.break_body } else { self.ubody };
                if let Some(body) = db.hashes.get_obody(b).or_else(|| db.hashes.get_obody(self.ubody)){
                    let same_gender = db.get_dress_gender(body.into()) == original_dress_gender;
                    if same_gender { result.body_model = body.into(); }
                    else if self.flag & 128 != 0 { result.body_model = body.into(); }
                }
                else if !engaged || (engaged && self.flag & 2 != 0) || (stun && self.flag & 32 != 0) {
                    let allow_cross_dress = self.flag & 128 != 0;
                    if let Some(body) = db.try_get_asset(AssetType::Body, b)
                        .or_else(|| db.try_get_asset(AssetType::Body, self.ubody))
                    {
                        let new_dress_gender = db.get_dress_gender(body.into());
                        if original_dress_gender == new_dress_gender { result.dress_model = body.into(); }
                        else if allow_cross_dress { result.dress_model = body.into(); }
                    }
                }
                for x in 0..5 {
                    if let Some(head) = db.try_get_asset(Acc(x as u8), self.acc[x]){
                        if head.contains("Msc0AT") { continue; }
                        else {
                            let oacc = head.replace("uAcc", "oAcc");
                            if db.hashes.o_acc.iter().any(|x| *x.1 == oacc) {
                                let accessory = new_asset_table_accessory(oacc.as_str(), ACC_LOC[x]);
                                result.commit_accessory(&accessory);
                            }
                        }
                    }
                }
            }
            if let Some(hair) = db.hashes.get_ohair(self.uhair){
                if (result.body_model.str_contains("Drg0AF_c05") || result.body_model.str_contains("Drg1AF_c05"))
                    && (hair.contains("h050") || hair.contains("h051")) { result.head_model = "oHair_h050".into();
                }
                else { result.head_model = hair; }
            }
            if let Some(ride) = result.ride_model.as_ref() {
                let current_mount = Mount::from(ride.to_string().as_str());
                let mount_index = i32::from(current_mount) - 1;
                if mount_index >= 0 {
                    if let Some(ride) = self.mount.get(mount_index as usize).and_then(|hash| db.hashes.get_mount_obody(*hash)) {
                        result.ride_model = Some(ride);
                    }
                }
            }
        }
    }
    pub fn try_load_from_file(dir_entry: &DirEntry, gender_restriction: Option<Gender>) -> Option<Self> {
        if let Ok(file) = read_to_string(dir_entry.path()) {
            let scale_name = SCALE_NAME.iter().map(|v| v.to_lowercase()).collect::<Vec<String>>();
            let color = COLORS.iter().map(|v| v.to_lowercase()).collect::<Vec<String>>();
            let var_names = VAR_NAMES.iter().map(|v| v.to_lowercase()).collect::<Vec<String>>();
            let mut lines = file.lines();
            let db = get_outfit_data();
            let mut out = PlayerOutfitData::new();
            while let Some(l) = lines.next() {
                if l.ends_with("=") || l.to_lowercase().ends_with("none") || !l.contains("=") { continue; }
                let spilt = l.split("=").collect::<Vec<&str>>();
                let lower = spilt[0].to_lowercase();
                let var_name = lower.as_str();
                if let Some(pos) = var_names.iter().position(|x| lower == *x){
                    if let Some(v) = if spilt.len() > 1 { db.try_get_asset_hash(spilt[1]) } else { None } {
                        match pos {
                            0 => { out.ubody = v; }
                            1 => { out.uhead = v; }
                            2 => { out.uhair = v; }
                            3..8 => { out.acc[pos - 3] = v; }
                            8..20 => {
                                let rel_pos = pos % 4;
                                if let Some(g) = db.get_aoc_gender_hash(rel_pos as i32, v){
                                    if g == Gender::Male { out.aoc[rel_pos] = v; }
                                    else if g == Gender::Male { out.aoc_alt[rel_pos] = v; }
                                }
                            }
                            20..25 => { out.mount[pos - 20] = v; }
                            25 => { out.voice = v; }
                            26 => { out.rig = v; }
                            _ => {}
                        }
                    }
                }
                else {
                    match var_name {
                        "flags" => { if let Ok(flag) = spilt[1].parse::<i32>() { out.flag = flag; } }
                        _ => {
                            if let Some((pos, value)) = scale_name.iter().position(|x| *x == var_name).zip(spilt[1].parse::<f32>().ok()) {
                                if value > 0.01 {
                                    if value >= 10.0 { out.scale[pos] = 1000; }
                                    else { out.scale[pos] = (value * 100.0 + 0.005) as u16; }
                                }
                            }
                            else if let Some(pos) = color.iter().position(|x| *x == var_name) {
                                let color = spilt[1].trim_start_matches("0x");
                                if let Ok(v) = u32::from_str_radix(color, 16) {
                                    out.colors[pos].values[0] = ((v >> 24) & 255) as u8;
                                    out.colors[pos].values[1] = ((v >> 16) & 255) as u8;
                                    out.colors[pos].values[2] = ((v >> 8) & 255) as u8;
                                    out.colors[pos].values[3] = (v & 255) as u8;
                                }
                                else {
                                    for x in [" ", ",", ":", "_", "-", ] {
                                        let count = spilt[1].split(x).count();
                                        if count >= 3 {
                                            let values: Vec<_> = spilt[1].split(x).map(|v| v.parse::<u8>().ok()).collect();
                                            let len = if values.len() >= 4 { 4 } else { values.len() };
                                            for x in 0..len {
                                                if let Some(v) = values[x] { out.colors[pos].values[x] = v; }
                                            }
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if out.is_empty(gender_restriction) { None } else { Some(out) }
        }
        else { None }
    }
    pub fn to_string(&self, hash: i32) -> String {
        if let Some(mut string) = PersonData::try_get_hash(hash).map(|v| format!("PID={} [{}]", v.pid, Mess::get_name(v.pid)))
            .or_else(|| GodData::try_get_hash(hash).map(|v| format!("GID={} [{}]", v.gid, Mess::get(v.mid))))
        {
            let none = "none".into();
            string.push_str(format!("\nFlags={}\n", self.flag).as_str());
            let db = OUTFIT_DATA.get_or_init(||OutfitData::init());
            if let Some(g) = db.get_dress_gender_hash(self.ubody) {
                string.push_str("Gender=");
                if g == Gender::Male { string.push_str("Male\n"); } else { string.push_str("Female\n"); }
            }
            string.push_str(format!("{}={}\n", VAR_NAMES[0], db.try_get_asset(AssetType::Body, self.ubody).unwrap_or(&none)).as_str());
            string.push_str(format!("{}={}\n", VAR_NAMES[1], db.try_get_asset(AssetType::Head, self.uhead).unwrap_or(&none)).as_str());
            string.push_str(format!("{}={}\n", VAR_NAMES[2], db.try_get_asset(AssetType::Hair, self.uhair).unwrap_or(&none)).as_str());
            for x in 0..5 {
                string.push_str(format!("{}={}\n", VAR_NAMES[3+x], db.try_get_asset(Acc(x as u8), self.acc[x]).unwrap_or(&none)).as_str());
            }
            for x in 0..4 {
                string.push_str(format!("{}={}\n", VAR_NAMES[12+x], db.try_get_asset(AssetType::AOC(x as u8), self.aoc[x]).unwrap_or(&none)).as_str());
                string.push_str(format!("{}={}\n", VAR_NAMES[16+x], db.try_get_asset(AssetType::AOC(x as u8), self.aoc_alt[x]).unwrap_or(&none)).as_str());
            }
            for x in 0..5 {
                string.push_str(format!("{}={}\n", VAR_NAMES[20+x], db.try_get_asset(AssetType::Mount(x as u8), self.mount[x]).unwrap_or(&none)).as_str());
            }
            string.push_str(format!("{}={}\n", VAR_NAMES[21], db.try_get_asset(AssetType::Voice, self.voice).unwrap_or(&none)).as_str());
            string.push_str(format!("{}={}\n", VAR_NAMES[22], db.try_get_asset(AssetType::Rig, self.rig).unwrap_or(&none)).as_str());
            for x in 0..16 { string.push_str(format!("{}={}\n", SCALE_NAME[x],  (self.scale[x] as f32) / 100.0).as_str()); }
            for x in 0..8 {
                string.push_str(COLORS[x]);
                string.push('=');
                string.push_str(format!("{} {} {} {}\n", self.colors[x].values[0], self.colors[x].values[1], self.colors[x].values[2], self.colors[x].values[3]).as_str());
            }
            string
        }
        else { String::new() }
    }
}

pub fn game_user_data_on_serialize(this: &GameUserData, stream: &mut Stream, _method_info: OptionalMethod){
    this.on_serialize(stream);
    stream.write_begin(UnitAssetData::version());
    let menu_data = UnitAssetMenuData::get();
    PLAYABLE_HASH.iter().for_each(|p|{
        if menu_data.data.iter().find(|v| v.person == *p).is_none() {
            menu_data.data.push(UnitAssetData::new_hash(*p, false)); }
    });
    let mut hash_map: HashSet<i32> = menu_data.data.iter().map(|v| v.person).collect();
    let _ = stream.write_int(hash_map.len() as i32).unwrap();
    println!("Serializing... {} Outfits Version: {}, ", menu_data.data.len(), UnitAssetData::version());
    menu_data.data.iter().for_each(|outfit| {
        if hash_map.contains(&outfit.person) {
            outfit.serialize(stream);
            hash_map.remove(&outfit.person);
        }
    });
    stream.write_end();
}
pub fn game_user_data_version(_this: &GameUserData, _method_info: OptionalMethod) -> i32 { crate::GAME_USER_DATA_VERSION }
pub fn game_user_data_on_deserialize(this: &GameUserData, stream: &mut Stream, version: i32, _method_info: OptionalMethod){
    this.on_deserialize(stream, version);
    let menu_data = UnitAssetMenuData::get();
    if !menu_data.is_loaded && version >= 21 {
        let version = stream.read_begin();
        if version < 6 { return; }
        let count = stream.read_int().unwrap_or(0);
        menu_data.data.clear();
        println!("Deserializing... {} Outfits", count);
        for _ in 0..count {
            let data = UnitAssetData::deserialize(stream, version);
            menu_data.add_data(data);
        }
        stream.read_end(true);
        PLAYABLE_HASH.iter().for_each(|p|{
            if menu_data.data.iter().find(|v| v.person == *p).is_none() {
                menu_data.data.push(UnitAssetData::new_hash(*p, false)); }
        });
        crate::capture::reset_faces(false);
        menu_data.is_loaded = true;
    }
}