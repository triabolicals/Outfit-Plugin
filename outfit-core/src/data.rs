use std::collections::HashMap;
pub use engage::{
    unit::Unit,
    gamedata::{accessory::AccessoryData, assettable::AssetTable, Gamedata, GodData, JobData, PersonData},
    mess::Mess,
    resourcemanager::*,
};
use std::io::{Cursor, Read};
// use std::fs::File;

// use assets::*;
pub use super::*;

mod color;
mod body;
mod acc;
mod hashes;
mod item;
pub mod anim;
pub(crate) mod dress;
pub mod unit_acc;
mod util;

pub use acc::*;
pub use body::*;
pub use color::*;
pub use hashes::*;
pub use item::*;
pub use unit_acc::AccessoryConditions;

use engage::gamedata::assettable::{AssetTableResult};
use engage::gamevariable::GameVariableManager;
use engage::random::Random;
use anim::AnimData;
use crate::data::dress::{DressData, JobDressData};
use crate::enums::Mount;

pub const KINDS: [&str; 8] = ["uBody_", "uHead_", "uHair_", "uAcc_spine2_Hair", "uAcc_head_", "uAcc_spine", "uAcc_Eff", "uAcc_shield_"];
const UNIQUES_JOBS: &[&str] = &[
    "Avn0", "Flr0", "Scs0", "Trl0", "Lnd0", "Slp0", "Cpd0", "Pcf0", //8 Base
    "Sds0A", "Sdp0A", "Drg0AM_c002", "Drg0AF_c052", "Drg1AF_c555", "Msn0D", // Char Specific    //8-13
    "Avn1", "Flr1", "Scs1", "Trl1", "Lnd1", "Slp1", "Cpd1", "Pcf1", //14 - 22   Promoted
    "Drg1AM_c001", "Drg1AF_c051", "Drg0AM_c001", "Drg0AF_c051", //22, 23
    "Dnc0AM", "Sdk0AM",
];
const AOC: [&str; 4] = ["Info", "Talk", "Demo", "Hub"];
#[derive(Default)]
pub struct AssetGroup {
    pub label_index: i32,
    pub flag: i32,
    pub ubody: Option<String>,
    pub uhead: Option<String>,
    pub uhair: Option<String>,
    pub aoc: [Option<String>; 4],
    pub acc: [Option<String>; 10],
    pub extra_body: Vec<String>,
}

impl AssetGroup {
    pub fn new(label_index: i32, flag: i32) -> Self {
        Self {
            label_index, flag,
            ..Default::default()
        }
    }
    pub fn to_string(&self) -> String {
        let mut out = format!("{}\t{}\t", self.label_index, self.flag);
        out += self.ubody.as_ref().map(|v| v.trim_start_matches("uBody_")).unwrap_or("-");
        out += "\t";
        out += self.uhead.as_ref().map(|v| v.trim_start_matches("uHead_")).unwrap_or("-");
        out += "\t";
        out += self.uhair.as_ref().map(|v| v.as_str()).unwrap_or("-");
        /*
        for x in 0..5 {
            out += "\t";
            out += self.acc[x].as_ref().map(|v| v.as_str()).unwrap_or("-");
        }
        */
        for x in 0..4 {
            out += "\t";
            out += self.aoc[x].as_ref().map(|v| v.as_str()).unwrap_or("-");
        }
        for x in 0..10 {
            out += "\t";
            out += self.acc[x].as_ref().map(|v| v.as_str()).unwrap_or("-");
        }
        self.extra_body.iter().for_each(|v| {
            out += "\t";
            out += v.as_str();
        });
        out
    }
}

#[derive(Default)]
pub struct OutfitList {
    pub character_body_list: GenderAssetList,
    pub class: ClassBodyList,
    pub engaged: GenderAssetList,
    pub other_outfits: GenderAssetList,
    pub class_male: Vec<ClassBodyList>,
    pub class_female: Vec<ClassBodyList>,
    pub hair_list: Vec<OutfitEntry>,
    pub head_list: Vec<OutfitEntry>,
    pub mount: [Vec<i32>; 5],
    pub acc: [Vec<AccEntry>; 5],
    pub aoc_info_m: [Vec<i32>; 4],
    pub aoc_info_f: [Vec<i32>; 4],
    pub voice: Vec<OutfitEntry>,
    pub skin: HashMap<i32, i32>,
    pub color_presets: Vec<ColorPreset>,
}
impl OutfitList {
    pub fn add_skin(&mut self, head: i32, skin_str: &str) {
        let s: Vec<_> = skin_str.split(",").flat_map(|x| x.parse::<i32>()).collect();
        if s.len() == 3 {
            let skin_value = s[0] | (s[1] << 8) | (s[2] << 16);
            self.skin.insert(head, skin_value);
        }
    }
    pub fn add_aoc(&mut self, hash: i32, kind: i32, female: bool) {
        if kind < 4 {
            if female { self.aoc_info_f[kind as usize].push(hash); }
            else { self.aoc_info_m[kind as usize].push(hash); }
        }
    }
    pub fn add(&mut self, kind: i32, hash: i32, female: bool, mid: impl AsRef<str>, flag: i32) {
        match kind {
            0 => { self.character_body_list.add(hash, female, mid, flag); }
            1 => { self.engaged.add(hash, female, mid, flag); }
            3 => {
                self.other_outfits.add(hash, female, mid, flag);
            }
            4 => {
                let smid = mid.as_ref().to_string();
                let count = self.head_list.iter().filter(|x| x.label.flags & 32 != 0 && smid == x.label.name).count() as i32;
                self.head_list.push(OutfitEntry::new(hash, mid, flag, count));
            }
            5 => { self.hair_list.push(OutfitEntry::new_with_flags(hash, mid, flag)); }
            9..14 => { self.add_aoc(hash, kind-9, female); }
            _ => { println!("Did not add Hash: {}", hash)}
        }
    }
    pub fn add_acc(&mut self, asset: &str, hash: i32, label: Option<&str>, body: Option<&str>, flag: i32) {
        if let Some(kind) = KINDS.iter().position(|k| asset.contains(*k)).filter(|k| *k >= 4).map(|k| k - 4) {
            let count = label.map(|label|
                self.acc[kind].iter()
                    .filter(|x| x.label.as_ref().is_some_and(|l| l == label))
                    .count()
            ).unwrap_or(0) as i32;

            self.acc[kind].push(AccEntry::new(hash, label, asset, body, flag, count));
        }
    }
    pub fn add_class(&mut self, mid: impl AsRef<str>, flag: i32, body: impl AsRef<str>, female: bool) {
        if female { self.class_female.push(ClassBodyList::new(mid, flag, body)); }
        else { self.class_male.push(ClassBodyList::new(mid, flag, body)); }
    }
    pub fn add_mount(&mut self, mount: impl AsRef<str>, hash: i32) {
        let index = Mount::from(mount.as_ref()) as i32 - 1;
        if index >= 0 && index < 5 {
            if !self.mount[index as usize].contains(&hash) { self.mount[index as usize].push(hash); }
        }
    }
    pub fn add_unique_class_body(&mut self, hash: i32, flags: i32, female: bool) {
        if female { self.class_female[0].list.push(ClassBodyEntry { hash, flags }); }
        else { self.class_male[0].list.push(ClassBodyEntry { hash, flags }); }
    }
    pub fn add_class_body(&mut self, hash: i32, body: &str, female: bool, flags: i32) {
        let set = if female { &mut self.class_female } else { &mut self.class_male };
        if let Some(class) = set.iter_mut().find(|x| body.contains(x.body_label.as_str())){
            class.list.push(ClassBodyEntry { hash, flags});
        }
    }
}
#[derive(Default)]
pub struct OutfitLabelTable {
    pub suffixes: HashMap<&'static str, OutfitLabel>,
    pub body: HashMap<&'static str, OutfitLabel>,
    pub asset: HashMap<i32, OutfitLabel>,
}
impl OutfitLabelTable {
    pub fn get_suffix(&self, asset: &str) -> Option<(&&'static str, &OutfitLabel)>{
        self.suffixes.iter().find(|s| asset.ends_with(s.0))
            .or_else(||self.suffixes.iter().find(|s| asset.ends_with(s.0)))
    }
    pub fn get_body(&self, asset: &str) -> Option<(&&str, &OutfitLabel)> {
        self.body.iter().find(|s| asset.contains(s.0))
    }
    pub fn get_suffix_name(&self, asset: &str) -> Option<(&'static Il2CppString, &'static str)> {
        self.suffixes.iter().find(|s| asset.ends_with(s.0)).map(|v| (v.1.get(), *v.0))
            .or_else(|| self.suffixes.iter().find(|s| asset.contains(*s.0))
                .map(|v|{
                    let last = asset.split(v.0).last().unwrap();
                    (format!("{} {}", v.1.get(), last).into(), *v.0)
                })
            )
    }
    pub fn get_body_name(&self, asset: &str) -> Option<(&'static Il2CppString, &'static str)>  {
        self.body.iter()
            .find(|s| asset.contains(*s.0))
            .map(|v| (v.1.get(), *v.0) )
    }
    pub fn try_get(&self, value: &str) -> Option<(&'static Il2CppString, &'static str)> {
        self.get_suffix_name(value).or_else(|| self.get_body_name(value))
    }
}

pub struct OutfitData {
    pub hashes: OutfitHashes,
    pub accessory_conditions: AccessoryConditions,
    pub list: OutfitList,
    pub labels: OutfitLabelTable,
    pub dress: DressData,
    pub item: Vec<ItemAsset>,
    pub anims: AnimData,
}

impl OutfitData {
    pub fn init() -> Self {
        let mut labels = OutfitLabelTable::default();
        let mut list = OutfitList::default();
        let mut hashes = OutfitHashes::default();
        AssetTable::get_list().unwrap().iter()
            .filter_map(|x| x.voice)
            .for_each(|v| {
                let hash = v.get_hash_code();
                if !hashes.voice.contains_key(&hash) {
                    list.voice.push(OutfitEntry{ hash, label: OutfitLabel{ name: v.to_string(), flags: -1, is_class: false, count: 0 } });
                    hashes.voice.insert(hash, v.to_string());
                }
            });
        // Color Presets
        let data = include_bytes!("../data/color.bin");
        let size = data.len() as u64;
        let mut color = Cursor::new(data);
        let mut buff: [u8; 26] = [0; 26];
        while color.position() < size {
            if color.read(&mut buff).is_ok() {
                let str_length = buff[0] - 25;
                let mut label_buff = vec![0; str_length as usize];
                color.read_exact(&mut label_buff).unwrap();
                let label = String::from_utf8(label_buff).unwrap();
                let mut colors: [i32; 8] = [0; 8];
                let mut count = 0;
                for x in 0..8 {
                    let mut color = 0;
                    for y in 0..3 { color += (buff[2+3*x+y] as i32) << 8*y; }
                    if color > 0 { count += 1; }
                    colors[x] = color;
                }
                let engaged = buff[1] & 1 != 0;
                list.color_presets.push(ColorPreset{ colors, engaged, count, label, });
            }
        }
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

        hashes.o_hair = assets.extract_if( .., |s| s.contains("oHair_h") || s.contains("oHair_dummy")).map(|s| (Il2CppString::new(s.as_str()).get_hash_code(), s)).collect();
        hashes.o_body = assets.extract_if(..,|s| s.contains("oBody_")).map(|s| (Il2CppString::new(s.as_str()).get_hash_code(), s)).collect();
        hashes.o_acc = assets.extract_if(.., |s| s.contains("oAcc_")).map(|s| (Il2CppString::new(s.as_str()).get_hash_code(), s)).collect();
        /*
        if let Ok(mut file) = std::fs::File::options().create(true).write(true).truncate(true).open("sd:/Outfits/dress.txt"){
            assets.iter().for_each(|i|{
                writeln!(&mut file, "{}", i).unwrap();
            });
        }
        */
        // Null Accessories
        let null_hash = Il2CppString::new("uAcc_head_null").get_hash_code();
        hashes.acc.insert(null_hash, "uAcc_head_null".into());
        list.acc[0].push(AccEntry{ hash: null_hash, label: Some("MID_SYS_None".to_string()), body: None, acc_suffix: None, acc_obj: String::new(), flag: 128, count: 0, });
        let null_hash = Il2CppString::new("null").get_hash_code();
        hashes.acc.insert(null_hash, "null".into());
        for x in 1..4 { list.acc[x].push(
            AccEntry{ hash: null_hash, label: Some("MID_SYS_None".to_string()), body: None, acc_suffix: None, acc_obj: String::new(), flag: 128, count: 0, }); }
        assets.extract_if(.., |s| s.contains("uBody") && (s.contains("DR_c") || s.contains("ER_c") || s.contains("BR_c") || s.contains("FR_c") || s.contains("CR_c")) && !s.contains("Fld"))
            .for_each(|str|{ list.add_mount(str.as_str(), hashes.add_ride_model(str.as_str())); });

        for mount in ["uBody_Wlf0CT_c751", "uBody_Wlf0CT_c707", "uBody_Fyd0DT_c707", "uBody_Fyd0DT_c715"]{ list.add_mount(mount, hashes.add_ride_model(mount)); }
        list.add_class("MID_SYS_SpecialPosition", 32, "____", false);
        list.add_class("MID_SYS_SpecialPosition", 32, "____", true);
        include_str!("../data/heads.txt").lines()
            .enumerate()
            .map(|(i, x)| (i & 1 != 0, x.split_whitespace()))
            .for_each(|(female, suffix)|{
                suffix.for_each(|ss|{
                    AOC.iter().enumerate().for_each(|(i, a)|{
                        assets.extract_if(.., |s| s.contains(format!("AOC_{}", a).as_str()) && s.contains(ss) && !s.contains("Photo") && !s.contains("Refresh"))
                            .for_each(|s|{
                                let hash = hashes.add_aoc(s.as_str());
                                list.add_aoc(hash, i as i32, female);
                            });
                    });
            });
        });
        // Playable Unit Casual Outfits
        EMBLEM.iter().for_each(|(i, d)| {
            let mpid = format!("MPID_{}",i);
            if let Some(s) = get_remove(&mut assets, format!("uBody_{}1AM", d).as_str()) {
                let hash = hashes.add_body(s.as_str(), false);
                list.add(1, hash, false, mpid.clone(), 32);
            }
            if let Some(s) = get_remove(&mut assets, format!("uBody_{}1AF", d).as_str()) {
                let hash = hashes.add_body(s.as_str(), true);
                list.add(1, hash, true, mpid, 32);
            }
        });
        let hair_append = ["e", "h", "n", "k"];
        let mut section = 0;
        [("File", "Filene"), ("Brod","Brodia"), ("Irci","Ircion") , ("Solu","Solum") , ("Lith", "Lithos")]
            .iter().enumerate()
            .for_each(|(_, x)|{
                ["M", "F"].iter().enumerate().map(|(i, g)|(i ==1, g)).for_each(|(female, gen)|{
                    for y in 0..6 {
                        let i = y+1;
                        let label_i = (y % 3) + 1;
                        if let Some(asset) = get_remove(&mut assets, format!("uBody_{}{}{}_c000", x.0, i, gen).as_str()){
                            let label = format!("MAID_{}{}{}{}", x.1, if y < 3 {"Formal"} else { "Casual"}, label_i, gen);
                            let hash = hashes.add_body(asset.as_str(), female );
                            list.add(3, hash, female , label, 32);
                        }
                    }
            });
        });
        include_str!("../data/labels.txt").lines()
            .for_each(|line|{
                if line.starts_with("END") { section += 1; }
                else {
                    let mut line = line.split_whitespace();
                    match section {
                        0 => {
                            let label = line.next().unwrap();
                            let name = line.next().unwrap().to_string();
                            let flags = line.next().and_then(|p| p.parse::<i32>().ok()).unwrap_or(0);
                            labels.body.insert(label, OutfitLabel::new_class(name.as_str(), flags));
                        }
                        1 => {
                            let label = line.next().unwrap();
                            let name = line.next().unwrap().to_string();
                            let flags = line.next().and_then(|p| p.parse::<i32>().ok()).unwrap_or(0);
                            let gender = line.next().and_then(|p| p.parse::<i32>().ok()).unwrap_or(0);
                            if gender > 0 && flags & 64 == 0 {
                                if gender & 1 != 0 { list.add_class(name.as_str(), flags, label, false); }
                                if gender & 2 != 0 { list.add_class(name.as_str(), flags, label, true); }
                            }
                            labels.body.insert(label, OutfitLabel::new_class(name.as_str(), flags));
                            let mut count_aoc = 0;
                            while let Some(suffix) = line.next() {
                                let female = count_aoc % 2 == 1;
                                if let Ok(num) = suffix.parse::<i32>() {
                                    let flag = if num > 700 { 36 } else { 32 } + if female { 2 } else { 1 };
                                    let suffix_name = if flags & 32 != 0 { name.clone() } else { format!("MJID_{}", name) };
                                    labels.suffixes.insert(suffix, OutfitLabel::new_mid(suffix_name, flag));
                                    let aoc = format!("AOC_Info_c{}", num);
                                    if get_remove(&mut assets, aoc.as_str()).is_some() {
                                        let hash = hashes.add_aoc(aoc.as_str());
                                        list.add_aoc(hash, 0, female);
                                    }
                                }
                                count_aoc += 1;
                            }
                        }
                        2 => {
                            let label = line.next().unwrap();
                            let name = line.next().unwrap().to_string();
                            let flags = line.next().and_then(|p| p.parse::<i32>().ok()).unwrap_or(0);
                            let person_label = OutfitLabel { name: name.clone(), flags, is_class: false, count: 0 };
                            if let Some(voice) = list.voice.iter_mut().find(|x| x.label.name == label && x.label.flags == -1 ){
                                voice.label = person_label.clone();
                            }
                            labels.suffixes.insert(label, person_label.clone());
                            let gender = line.next().and_then(|p| p.parse::<i32>().ok()).unwrap_or(0);
                            let generic = label.parse::<i32>().map(|v| v >= 800).unwrap_or(false);
                            if gender != 0 {
                                let female = gender == 2;
                                let mut skin: Option<String> = None;
                                let mut ohair: Option<&str> = None;
                                let mut labeled_assets = vec![];
                                let mut skin_body = false;
                                while let Some(a) = line.next() {
                                    if a.contains("skin=") { skin = a.split("=").last().map(|s| s.to_string()); }
                                    else if a == "skinb" { skin_body = true; }
                                    else if a == "Wear" {
                                        if let Some(asset) = get_remove(&mut assets, format!("Wear{}{}", if female { "F_c"} else { "M_c"}, label).as_str()) { labeled_assets.push(asset); }
                                    }
                                    else if a.contains("null") { labeled_assets.push(a.to_string()); }
                                    else if a.contains("oHair") { ohair = a.split("_h").last(); }
                                    else if let Some(asset) = get_remove(&mut assets, a) { labeled_assets.push(asset); }
                                    else if a.starts_with("AOC_Info") {
                                        if let Some(hash) = hashes.aoc.iter().find(|x| *x.1 == a).map(|x| *x.0){ labels.asset.insert(hash, person_label.clone()); }
                                    }
                                    else if a.starts_with("voice=") {
                                        let voiceline = a.trim_start_matches("voice=");
                                        if let Some(voice) = list.voice.iter_mut().find(|x| x.label.name == voiceline && x.label.flags == -1 ){
                                            voice.label = person_label.clone();
                                            if voiceline.ends_with("_E") { voice.label.flags |= 4; }
                                        }
                                    }
                                    else if a.starts_with("voice2=") {
                                        let voice = a.trim_start_matches("voice2=");
                                        if let Some(voice) = list.voice.iter_mut().find(|x| x.label.name == voice && x.label.flags == -1 ){
                                            voice.label = person_label.clone();
                                            voice.label.flags |= 16;
                                        }
                                    }
                                }
                                assets.extract_if(.., |s| !s.contains("Hair") && !s.contains("uBody") && (s.ends_with(label) || (s.contains(label) && s.contains("uAcc"))))
                                    .for_each(|s|{ labeled_assets.push(s.to_string()); });
                                if !generic {
                                    assets.extract_if(.., |s| s.contains("Hair") && (s.ends_with(label) || (s.contains(label) && hair_append.iter().any(|a| s.ends_with(a)))))
                                        .for_each(|s|{ labeled_assets.push(s.to_string()); });
                                }
                                labeled_assets.iter().for_each(|s| {
                                    if s.starts_with("uBody") && (s.contains("M_c") || s.contains("F_c")) {
                                        let hash = hashes.add_body(s.as_str(), female);
                                        if let Some(pos) = UNIQUES_JOBS.iter().position(|u| s.contains(u)) {
                                            let flags2 =
                                                match pos {
                                                    0..8 => { 3 }
                                                    8..14 => { 5 }
                                                    _ => { 1 }
                                                };
                                            list.add_unique_class_body(hash, flags2, female);
                                            if pos >= 14 && pos <= 23 { list.add(0, hash, female, name.as_str(), flags|16); }   // Promoted
                                            else { list.add(0, hash, female, name.as_str(), flags); }
                                        }
                                        else if EMBLEM.iter().enumerate()
                                            .any(|(i, a) | s.contains(a.1) && (i < 12 || i == 23)  ) && s.ends_with("c000")
                                        {
                                            let amiibo_flag = flags | 2048;
                                            list.add(0, hash, female, name.as_str(), amiibo_flag);
                                        }
                                        else if s.contains("Wear") { list.add(0, hash, female, name.as_str(), flags|128); }
                                        else {
                                            list.add_class_body(hash, s, female, flags);
                                            list.add(0, hash, female, name.as_str(), flags);
                                        }
                                        if skin_body {
                                            if let Some(skin) = skin.as_ref() {
                                                list.add_skin(hash, skin.as_str());
                                            }
                                        }
                                    }
                                    else if s.starts_with("uHair") ||s.contains("_Hair"){
                                        let label = if generic { Some(label) } else { ohair };
                                        let hash = hashes.add_hair(s.as_str(), label);
                                        let mut hair_flag = flags;
                                        if s.ends_with("e") { hair_flag |= 8192 }
                                        else if s.ends_with("k") { hair_flag |= 16384 }
                                        else if s.ends_with("h") { hair_flag |= 128 }
                                        else if s.ends_with("n") { hair_flag |= 16 }
                                        list.add(5, hash, female, name.as_str(), hair_flag);
                                    }
                                    else if s.starts_with("uHead") {
                                        let hash = hashes.add_head(s.as_str());
                                        list.add(4, hash, female, name.as_str(), flags);
                                        if let Some(skin) = skin.as_ref() { list.add_skin(hash, skin.as_str()); }
                                    }
                                    else if s.starts_with("uAcc") && !s.contains("Hair") {
                                        let hash = hashes.add_acc(s.as_str(), None);
                                        let l = if label.len() > 3 { &label[0..3] } else { label };
                                        let acc_name = if flags & 32 == 0 { &format!("MPID_{}", name) } else { &name };
                                        list.add_acc(s.as_str(), hash, Some(acc_name), Some(l), 0);
                                    }
                                });
                            }
                        }
                        3|4 => {
                            let female = section == 4;
                            let body = format!("uBody_{}", line.next().unwrap());
                            let label = line.next().map(|v| v.to_string()).unwrap();
                            let flags = 32 | line.next().and_then(|p| p.parse::<i32>().ok()).unwrap_or(0);
                            assets.extract_if(.., |s| s.contains(body.as_str()))
                                .for_each(|other_outfit|{ list.add(3, hashes.add_body(other_outfit, female), female, label.as_str(), flags); });
                        }
                        5 => {
                            let head = line.next().unwrap();
                            let flag = line.next().and_then(|p| p.parse::<i32>().ok()).unwrap_or(0);
                            let label = line.next().unwrap().to_string();
                            if get_remove(&mut assets, head).is_some() {
                                let hash = hashes.add_head(head);
                                list.add(4, hash, false, label.as_str(), flag);
                                if let Some(ohair) = line.next().map(|s| Il2CppString::new(s).get_hash_code()){
                                    if hashes.o_hair.contains_key(&ohair){ hashes.head_hair.insert(hash, ohair); }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            });
        list.class_male.iter_mut().skip(1).for_each(|class| {
            assets.extract_if(.., |s| s.contains(format!("uBody_{}", class.body_label).as_str()) && s.contains("M_c"))
                .for_each(|s| {
                    let flags = if s.contains("c_70") { 4 } else { 0 };
                    let hash = hashes.add_body(s.as_str(), false);
                    class.list.push(ClassBodyEntry{ hash, flags });
                });
        });
        list.class_female.iter_mut().skip(1).for_each(|class| {
            assets.extract_if(.., |s| s.contains(format!("uBody_{}", class.body_label).as_str()) && s.contains("F_c"))
                .for_each(|s| {
                    let flags = if s.contains("c_70") { 4 } else { 0 };
                    let hash = hashes.add_body(s.as_str(), true);
                    class.list.push(ClassBodyEntry{ hash, flags });
                });
        });
        let kinds = ["ubody_", "uhead_c", "uhair_h", "uacc_spine2_hair", "uacc_head_", "uacc_spine", "uacc_eff", "uacc_shield_"];
        let female = ["f_c", "f1_c", "f2_c", "f3_c", "f4_c"];
        let male = ["m_c", "m1_c", "m2_c", "m3_c", "m4_c"];
        let mut remove = vec![];
        include_str!("../data/other.txt").lines().map(|l| l.split_whitespace())
            .for_each(|mut l| {
                let search = l.next().unwrap();
                let mut label = l.next().unwrap().to_string();
                let mut flag = l.next().and_then(|l| l.parse::<i32>().ok()).unwrap_or(0);
                assets.extract_if(.., |s| s.contains(search) && KINDS.iter().any(|x| s.contains(*x)))
                    .for_each(|asset|{
                        let kind = KINDS.iter().position(|x| asset.contains(*x)).unwrap();
                        match kind {
                            0 => {
                                if search.contains("uBody_") {
                                    list.add(0, hashes.add_head(asset.as_str()), search.contains("F_c"), label.as_str(), flag);
                                }
                            }
                            1 => {  //Head
                                if label.contains("NPC") && asset.contains("_c")
                                    { label = format!("NPC {}", asset.split("_c").last().unwrap()); }
                                if asset.ends_with("s") { flag |= 256; }
                                list.add(4, hashes.add_head(asset.as_str()), false, label.as_str(), flag);
                            }
                            2 => {
                                if label.contains("NPC") && asset.contains("_h")
                                { label = format!("NPC {}", asset.split("_h").last().unwrap()); }
                                list.add(5, hashes.add_hair(asset.as_str(), None), false, label.as_str(), flag);
                            }
                            3 => {
                                if label.contains("NPC") && asset.contains("_Hair") {
                                    label = format!("NPC {}", asset.split("Hair").last().unwrap());
                                }
                                list.add(5, hashes.add_hair(asset.as_str(), None), false, label.as_str(), flag);
                            }
                            4..8 => {
                                let hash = hashes.add_acc(asset.as_str(), Some(kind as i32 - 4));
                                list.add_acc(asset.as_str(), hash, Some(label.as_str()), None, flag);
                            }
                            _ => { println!("Ignored: {}", asset); }
                        }
                    })
            });

        assets.iter().enumerate()
            .filter(|(_, s)|{
                let lower = s.to_lowercase();
                !s.contains("null") && kinds.iter().any(|k| lower.contains(*k))
            })
            .for_each(|(i, asset)| {
                let lower = asset.to_lowercase();
                if let Some(kind) = kinds.iter().position(|k| lower.contains(*k)){
                    match kind {
                        0 => {
                            if !lower.contains("t_c") && !lower.contains("r_c") {
                                if let Some((condition, gender)) = find_entries_with_model_field(
                                    2, asset,
                                    |entry, asset| entry.dress_model.is_some_and(|s| s.str_contains(asset)))
                                    .iter()
                                    .flat_map(|&i| AssetTable::try_index_get(i))
                                    .find_map(|a| get_aid_condition(a))
                                {
                                    remove.push(i);
                                    let name = get_asset_name(&condition, gender).unwrap_or(asset.to_string());
                                    if gender != Gender::None {
                                        let female = gender == Gender::Female;
                                        let hash = hashes.add_body(asset.as_str(), female);
                                        list.add(3, hash, female, name.as_str(), 32);
                                    } else {
                                        if male.iter().any(|&s| lower.contains(s)) {
                                            list.add(3, hashes.add_body(asset.as_str(), false), false, name.as_str(), 32);
                                        }
                                        else if female.iter().any(|&s| lower.contains(s)) {
                                            list.add(3, hashes.add_body(asset.as_str(), true), true, name.as_str(), 32);
                                        }
                                        else {
                                            let hash = hashes.add_body(asset.as_str(), true);
                                            hashes.add_body(asset.as_str(), false);
                                            list.add(3, hash, true, name.as_str(), 32);
                                            list.add(3, hash, false, name.as_str(), 32);
                                        }
                                    }
                                }
                            }
                            else { remove.push(i); }
                        }
                        1 => {
                            if let Some((condition, gender)) = find_entries_with_model_field(
                                2, asset,
                                |entry, asset| entry.head_model.is_some_and(|s| s.str_contains(asset)))
                                .iter()
                                .flat_map(|&i| AssetTable::try_index_get(i))
                                .find_map(|a| get_aid_condition(a))
                            {
                                let name = get_asset_name(&condition, gender).unwrap_or(asset.to_string());
                                let hash = hashes.add_head(asset.as_str());
                                list.add(4, hash, false, name.as_str(), 32);
                                remove.push(i);
                            }
                        }
                        2 => {
                            if let Some((condition, gender)) =
                                find_entries_with_model_field(
                                    2, asset,
                                    |entry, asset| entry.hair_model.is_some_and(|s| s.str_contains(asset)))
                                    .iter()
                                    .flat_map(|&i| AssetTable::try_index_get(i))
                                    .find_map(|a| get_aid_condition(a))
                            {
                                let name = get_asset_name(&condition, gender).unwrap_or(asset.to_string());
                                let hash = hashes.add_hair(asset.as_str(), None);
                                list.add(5, hash, false, name.as_str(), 32);
                                remove.push(i);
                            }
                        }
                        3 | 4 | 5 | 7 => {
                            let mut added = false;
                            if let Some((condition, gender)) = find_entries_with_model_field(
                                2, asset,
                                |entry, asset| entry.accessory_list.list.iter()
                                    .any(|a| a.model.is_some_and(|model| model.str_contains(asset))))
                                .iter()
                                .flat_map(|&i| AssetTable::try_index_get(i))
                                .find_map(|a| get_aid_condition(a))
                            {
                                if kind == 3 {  // Haur
                                    let name = get_asset_name(&condition, gender).unwrap_or(asset.to_string());
                                    let hash = hashes.add_hair(asset.as_str(), None);
                                    list.add(5, hash, false, name.as_str(), 32);
                                    remove.push(i);
                                    added = true;
                                }
                                else {
                                    let body = labels.get_body(asset);
                                    if let Some(name) = get_asset_name(&condition, gender) {
                                        let hash = hashes.add_acc(asset.as_str(), Some(kind as i32 - 4));
                                        list.add_acc(asset, hash, Some(&name), body.map(|v|  &**v.0), 0);
                                        remove.push(i);
                                        added = true;
                                    }
                                }
                            }
                            if kind > 3 && !added {
                                if let Some(v) = labels.get_body(asset){
                                    let name = if v.1.flags & 32 == 0 { &format!("MJID_{}", v.1.name) } else { &v.1.name };
                                    let hash = hashes.add_acc(asset.as_str(), Some(kind as i32 - 4));
                                    list.add_acc(asset, hash, Some(&name), Some(v.0), 0);
                                    remove.push(i);
                                }
                                else if let Some(v) = labels.get_suffix(asset) {
                                    let name = if v.1.flags & 32 == 0 { &format!("MPID_{}", v.1.name) } else { &v.1.name };
                                    let hash = hashes.add_acc(asset.as_str(), Some(kind as i32 - 4));
                                    list.add_acc(asset, hash, Some(&name), Some(v.0), 0);
                                    remove.push(i);
                                }
                            }
                        }
                        6 => {
                            let hash = hashes.add_acc(asset.as_str(), Some(kind as i32 - 4));
                            list.add_acc(asset, hash, None, None, 0);
                        }
                        _ => {}
                    }
                };
            });

        remove.iter().rev().for_each(|&pos| { assets.remove(pos); });
        remove.clear();
        assets.extract_if(.., |s|{
            let lower = s.to_lowercase();
            kinds.iter().any(|k| lower.contains(*k))
        }).for_each(|asset|{
            if let Some(kind) = kinds.iter().position(|k| asset.contains(*k)){
                let name = asset.trim_start_matches(kinds[kind]);
                match kind {
                    0 => {
                        if !asset.contains("R_c") && !asset.contains("T_c") {
                            hashes.add_body(asset.as_str(), false);
                            let hash = hashes.add_body(asset.as_str(), true);
                            list.add(3,hash, false, name, -1);
                            list.add(3,hash, true, name, -1);
                        }
                    }
                    1 => {
                        let hash = hashes.add_head(asset.as_str());
                        list.add(4, hash, false, name, -1);
                    }
                    2|3 => {
                        let hash = hashes.add_hair(asset.as_str(), None);
                        list.add(5, hash, false, name, -1);
                    }
                    _ => {
                        let hash = hashes.add_acc(asset.as_str(), Some(kind as i32 - 4));
                        list.add_acc(asset.as_str(), hash, None, None, 0);
                    }
                }
            };
        });

        let hash = hashes.add_acc("uBody_Msc0AT_c000", Some(3));
        list.acc[3].push(AccEntry::new(hash, Some("MPID_Mascot"), "uBody_Msc0AT_c000", None, 0, 0));
        let dress = DressData::init(&mut hashes, &mut list);
        let anims = AnimData::init(&mut assets);
        println!("Finish with Outfit data");
        Self {
            dress, anims, list, hashes, labels,
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
                data.apply(result, conditions.mode, dress_gender);
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
            // self.get_dress_gender(result.body_model) };
        if dress_gender != Gender::Male && dress_gender != Gender::Female { return; }
        let kind_ =
            if conditions.flags.contains(AssetFlags::CombatTranforming) { 9 }
            else if conditions.flags.contains(AssetFlags::Bullet) { 10 }
            else { conditions.kind };
        // println!("Correct Anim Kind: {}", kind_);
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
                    if !self.anims.has_anim(result, dress_gender, mount, conditions.mode, kind_) || (unit.person.get_job().is_some_and(|v| v.parent.hash == 499211320) && conditions.kind > 0) {
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
            if let Some(skin) = self.list.skin.get(head.0) {
                ColorPreset::set_color(&mut result.unity_colors[2], *skin);
            }
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
    pub fn try_get_suffix(&self, asset: &String) -> Option<(&'static Il2CppString, &'static str)> { self.labels.get_suffix_name(asset) }
    pub fn try_get_body_label(&self, asset: &String) -> Option<(&'static Il2CppString, &'static str)>  { self.labels.get_body_name(asset) }
    pub fn try_get_asset_label(&self, value: &String) -> Option<(&'static Il2CppString, &'static str)> { self.try_get_suffix(value).or_else(|| self.try_get_body_label(value)) }
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
    pub fn get_aoc_gender_hash(&self, ty: i32, hashcode: i32) -> Option<Gender> {
        if self.list.aoc_info_m.get(ty as usize).map(|x| x.contains(&hashcode)).unwrap_or(false) { Some(Gender::Male) }
        else if self.list.aoc_info_f.get(ty as usize).map(|x| x.contains(&hashcode)).unwrap_or(false) { Some(Gender::Female) }
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