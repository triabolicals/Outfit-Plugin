
use engage::gamedata::assettable::*;
use engage::gamedata::Gamedata;
use engage::gamedata::item::ItemData;
use engage::mess::Mess;

use unity::prelude::Il2CppString;

use crate::data::KINDS;
use crate::localize::MenuTextCommand;
/*
use bitflags::bitflags;
use engage::ut::Ut;
use crate::{AssetType, MenuText};
 */

#[derive(Clone, PartialEq, Default)]
pub struct OutfitLabel {
    pub name: String,
    pub flags: i32,
    pub is_class: bool,
    pub count: i32,
}
#[derive(Clone)]
pub struct OutfitEntry { pub hash: i32, pub label: OutfitLabel, }

impl OutfitEntry {
    pub fn new_mid(hash: i32, mid: impl AsRef<str>) -> Self { Self { hash, label: OutfitLabel::new_mid(mid, 32) } }
    pub fn new(hash:i32, mid: impl AsRef<str>, flags: i32, count: i32) -> Self { Self { hash, label: OutfitLabel::new_mid_count(mid, flags, count) } }
    pub fn new_with_flags(hash: i32, mid: impl AsRef<str>, flags: i32) -> Self { Self { hash, label: OutfitLabel::new_mid(mid, flags) } }
    pub fn get_name(&self) -> &'static Il2CppString { self.label.get() }
}
impl OutfitLabel {
    pub fn new_class(mid: impl AsRef<str>, flags: i32) -> Self { Self { name: mid.as_ref().to_string(), flags, is_class: true, count: 0, } }
    pub fn new_asset(asset: &String) -> Self { Self { name: asset.clone(), flags: -1, is_class: false, count: 0, } }
    pub fn new_mid(mid: impl AsRef<str>, flags: i32) -> Self { Self { name: mid.as_ref().to_string(), flags, is_class: false, count: 0,} }
    pub fn new_mid_count(mid: impl AsRef<str>, flags: i32, count: i32) -> Self { Self { name: mid.as_ref().to_string(), flags, is_class: false, count, } }
    pub fn get(&self) -> &'static Il2CppString {
        if self.flags == -1 {
            return
                if let Some(prefix) = KINDS.iter().find(|k| self.name.starts_with(**k)) {
                    self.name.as_str().trim_start_matches(prefix).into()
                }
                else if self.count > 0 { format!("{} {}", self.name, self.count+1).into() }
                else { self.name.as_str().into() }
        }
        if !self.is_class {
            let mut mpid =
                if self.flags & 8 != 0 { self.name.clone() }
                else if self.flags & 544 == 0 { Mess::get(format!("MPID_{}", self.name)).to_string() }
                else if self.flags & 544 == 32 { Mess::get(&self.name).to_string() }
                else {
                    self.name.parse::<i32>().ok()
                        .map(|x| MenuTextCommand::get_from_index(x).to_string())
                        .unwrap_or("???".to_string())
                };

            if self.flags & 2048 != 0 { // amiibo
                mpid.push_str(" (Amiibo)");
                return mpid.into();
            }
            if self.flags & 16384 != 0 { return format!("{} Kings", mpid).into() }
            let mut end = String::new();
            if self.flags & 1 != 0 { end.push_str("M"); }
            else if self.flags & 2 != 0 { end.push_str("F"); }
            if self.flags & 256 != 0 { end.push_str("S"); }
            if self.flags & 32 == 0 {
                if self.flags & 48 == 16 { end.push_str("2"); }
                else if self.flags & 80 == 16 { end.push_str("3"); }
            }
            else if self.count > 0 {
                end.push_str((self.count + 1).to_string().as_str());
            }
            if end.len() > 0 { mpid.push_str(format!(" {}", end).as_str()); }
            if self.flags & 128 != 0 { return format!("{} ({})", mpid, Mess::get("MID_SAVEDATA_SEQ_HUB")).into() }
            if self.flags & 8192 != 0 { return format!("{} ({})",  mpid, MenuTextCommand::Engage).into() }
            if self.flags & 4 != 0 {
                Mess::set_argument(0, mpid);
                return Mess::get("MPID_Morph_Prefix");
            }
            else if self.flags & 1024 != 0 {
                Mess::set_argument(0, mpid);
                return Mess::get("MPID_God_Prefix");
            }
            mpid.into()
        }
        else {
            if self.flags & 32 == 0 { Mess::get(format!("MJID_{}", self.name).as_str()) }
            else { Mess::get(&self.name) }
        }
    }
}
/*
bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct AssetItemFlags: i32 {
        const Male = 1;
        const Female = 2;
        const Morph = 4;
        const God = 8;
        const Somniel = 16;
        const Engaged =  32;
        const Amiibo = 64;
        const Base = 128;
        const Promoted = 256;
    }
}

#[derive(Clone)]
pub struct AssetItem {
    pub hash: i32,
    pub count: i32,
    pub kind: AssetType,
    pub flags: AssetItemFlags,
}
const AOC: [&str; 4] = ["Info", "Talk", "Demo", "Hub"];

impl AssetItem {
    const PARSE_KINDS: [&str; 17] =
        ["uBody_", "uHead_", "uHair_", "uAcc_spine2_Hair", "uAcc_head_", "uAcc_spine", "uAcc_Eff", "uAcc_shield_",
            "Info", "Talk", "Demo", "Hub", "BR_c", "CR_c", "DR_c", "ER_c", "FR_c"
        ];
    pub fn new<'a>(asset_name: impl Into<&'a Il2CppString>, flags: i32) -> Option<Self> {
        let str = asset_name.into();
        let str2 = str.to_string();

        let kind =
            Self::PARSE_KINDS.iter().position(|x| str2.contains(x)).map(|pos| {
                match pos {
                    0 => { AssetType::Body }
                    1 => { AssetType::Head }
                    2 | 3 => { AssetType::Hair }
                    4..8 => { AssetType::Acc((pos - 4) as u8) }
                    8..12 => { AssetType::AOC((pos - 8) as u8) }
                    _ => { AssetType::Mount((pos - 12) as u8 ) }
                }
            })?;

        let hash = str.get_hash_code();
        let flags = AssetItemFlags::from_bits(flags)?;
        Some(Self { hash, count: 0, kind, flags, })
    }
    pub fn try_add_voice(voice: &str, voice_list: &mut Vec<String>) -> Option<Self> {
        if let Some(pos) = voice_list.iter().position(|x| *x == voice) {
            let voice = voice_list.remove(pos);
            let hash = Ut::hash_fnv_string(voice);
            Some(AssetItem{ hash, count: 0, kind: AssetType::Voice, flags: AssetItemFlags::from_bits(0).unwrap() })
        }
        else { None }

    }
}

pub struct AssetGroup{
    pub label: &'static str,
    pub list: Vec<AssetItem>,
}
impl AssetGroup {
    pub fn new_character_group(line: &'static str, asset_list: &mut Vec<String>, voice_list: &mut Vec<String>) -> Option<Self> {
        let mut spilt = line.split_whitespace();
        let mut list = vec![];
        let label = spilt.next()?;
        if let Some((_, voice)) = label.split_once("_"){
            if let Some(voice_asset) = AssetItem::try_add_voice(voice, voice_list) {
                list.push(voice_asset);
            }
        }
        let mut current_suffix = String::new();
        let mut trimmed_suffix = String::new();
        let mut num = 0;
        let mut flag = 0;
        let mut dark: Option<String> = None;
        while let Some(s) = spilt.next() {
            let (s1, f) =
                s.split_once(":")
                    .map(|(s, f)| (s, f.parse::<i32>().ok().unwrap_or(0)))
                    .unwrap_or((s, 0));

            if s1.starts_with("c") {
                current_suffix = s.to_string();
                trimmed_suffix = s.trim_start_matches("c").to_string();
                num = trimmed_suffix.parse::<i32>().unwrap_or(0);
                if (num >= 510 && num < 517) || (num >= 530 && num < 538) || (num >= 560 && num < 567) || (num >= 580 && num < 588) {
                    dark = Some((num + 7).to_string());
                }
                flag = f;
                try_search_and_add(&mut list, f, asset_list, "uHead", None, Some(current_suffix.as_str()));
                try_search_and_add(&mut list, f, asset_list, "uHair", None, Some(current_suffix.replace("c", "h").as_str()));
                try_search_and_add(&mut list, f, asset_list, "AOC_Info_", None, Some(current_suffix.replace("c", "h").as_str()));
                try_search_and_add(&mut list, f|32, asset_list, "AOC_Info", None, Some(format!("{}_Eng", current_suffix).as_str()));
                try_search_and_add(&mut list, f, asset_list, "AOC_Demo_", None, Some(current_suffix.as_str()));
                try_search_and_add(&mut list, f, asset_list, "AOC_Talk_", None, Some(current_suffix.as_str()));
                if let Some(dark) = dark.as_ref(){
                    try_search_and_add(&mut list, f, asset_list, "uHead", None, Some(dark.as_str()));
                    try_search_and_add(&mut list, f, asset_list, "uHair", None, Some(dark.as_str()));
                }
            }
            else if current_suffix.is_empty() { continue; }
            else {
                if s1 == "w" { try_search_and_add(&mut list, f|16, asset_list, "uBody", Some("Wear"), Some(current_suffix.as_str())); }
                else if s1.starts_with("Wear") { try_search_and_add(&mut list, f|16, asset_list, "uBody", Some(s1), Some(current_suffix.as_str())); }
                else if s1 == "h" {
                    let flag =
                        if s1.ends_with("e") { 32 }
                        else if s1.ends_with("h"){ 16 }
                        else if s1.ends_with("k") { 256 }
                        else { 0 };

                    let hair = format!("Hair{}", trimmed_suffix);
                    try_search_and_add(&mut list, f|flag, asset_list, "uAcc_spine2", None, Some(hair.as_str()));
                    if dark.is_some() {
                        let hair = format!("Hair{}", num+7);
                        try_search_and_add(&mut list, f|flag, asset_list, "uAcc_spine2", None, Some(hair.as_str()));
                    }
                }
                else if let Some(voice_asset) = AssetItem::try_add_voice(s1, voice_list) { list.push(voice_asset); }
                else if try_search_and_add(&mut list, f, asset_list, "uBody", Some(s1), Some(current_suffix.as_str())) {
                    if let Some(dark) = dark.as_ref(){
                        try_search_and_add(&mut list, f|64, asset_list, "uBody", Some(s1), Some("c000"));
                        try_search_and_add(&mut list, f|8, asset_list, "uBody", Some(s1), Some(dark.as_str()));
                    }
                    try_search_and_add(&mut list, f|64, asset_list, "uBody", Some(s1), Some("c000"));
                    if f & 128 != 0 {
                        try_search_and_add(&mut list, f, asset_list, "uBody", Some(s1.replace("0", "1").as_str()), Some(current_suffix.as_str()));
                    }
                }
                else { try_search_and_add(&mut list, f, asset_list, "", Some(s1), None); }
            }
        }
        Some(Self { label, list })
    }
}

fn try_search_and_add(list: &mut Vec<AssetItem>, flag: i32, asset_list:  &mut Vec<String>, prefix: &str, body: Option<&str>, suffix: Option<&str>) -> bool {
    let body = body.unwrap_or("");
    let suffix = suffix.unwrap_or("");
    if let Some(pos) = asset_list.iter().position(|x| x.starts_with(prefix) && x.contains(body) && x.ends_with(suffix)){
        let str = asset_list.remove(pos);
        if let Some(asset) = AssetItem::new(str, flag) {
            list.push(asset);
            return true;
        }
    }
    false
}
*/

pub struct ItemAsset {
    pub hash: i32,
    pub entry: i32,
    pub kind: i32,
}
impl ItemAsset {
    pub fn init() -> Vec<Self> {
        ItemData::get_list().unwrap().iter().flat_map(|item| Self::from_item(item)).collect()
    }
    pub fn from_item(data: &ItemData) -> Option<Self> {
        let con = AssetTableStaticFields::get_condition_index(data.iid);
        let entry = AssetTableStaticFields::get().search_lists[2].iter().find(|x| x.condition_indexes.has_condition_index(con)).map(|entry| entry.parent.index)?;
        Some(Self { entry, hash: data.parent.hash, kind: data.kind as i32 })
    }
    pub fn apply(&self, result: &mut AssetTableResult) {
        if let Some(entry) = AssetTable::try_index_get(self.entry) { result.commit_asset_table(entry); }
    }
}