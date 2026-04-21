use bitflags::bitflags;
use engage::gamedata::assettable::*;
use engage::gamedata::Gamedata;
use engage::gamedata::item::ItemData;
use engage::mess::Mess;
use unity::prelude::Il2CppString;
use crate::{capitalize_first, get_remove, AssetLabelTable, AssetType, OutfitHashes};

const ACC: [&str; 10] = ["Band", "Dress", "Ear", "Glass", "Hat", "Kings", "Tiara", "Helm", "Shield", "Hood"];
bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
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
        const Band = 1 << 9;
        const Dress = 1 << 10;
        const Ear = 1 << 11;
        const Glass = 1 << 12;
        const Hat = 1 << 13;
        const Kings = 1 << 14;
        const Tiara = 1 << 15;
        const Helm = 1 << 16;
        const Shield = 1 << 17;
        const Hood = 1 << 18;
        const Playable = 1 << 20;
        const AccessoryShop = 1 << 30;
        const NoPhotograph = 1 << 31;
    }
}
impl AssetItemFlags {
    pub fn modify_name(&self, value: &String, count: i32) -> &'static Il2CppString {
        let mut s = value.clone();
        s = capitalize_first(s.as_str());
        if self.contains(AssetItemFlags::Male) { add_str(&mut s, "M"); }
        else if self.contains(AssetItemFlags::Female) { add_str(&mut s, "F") }

        if self.contains(AssetItemFlags::Base) { add_mess(&mut s, "MID_SYS_BasicPosition"); }
        else if self.contains(AssetItemFlags::Promoted) { add_mess(&mut s, "MID_SYS_SeniorPosition"); }
        else if self.contains(AssetItemFlags::Somniel) { add_mess(&mut s,"MID_Hub_Solanel"); }
        else if self.contains(AssetItemFlags::Engaged) { add_mess(&mut s, "MID_BGM_Evt_Engage_ST_Play"); }
        else if self.contains(AssetItemFlags::Amiibo) { add_str(&mut s, "amiibo"); }
        else if self.contains(AssetItemFlags::Playable) { add_mess(&mut s, "MID_MENU_ACHIEVEMENT_COUNT_UNIT"); }

        let bits = self.bits() >> 9;
        for x in 0..ACC.len() {
            if bits & (1 << x) != 0 {
                add_str(&mut s, ACC[x]);
                break;
            }
        }
        if count > 0 { add_str(&mut s, (count+1).to_string()); }
        if self.contains(AssetItemFlags::Morph) {
            Mess::set_argument(0, s);
            Mess::get("MPID_Morph_Prefix")
        }
        else if self.contains(AssetItemFlags::God) {
            Mess::set_argument(0, s);
            Mess::get("MPID_God_Prefix")
        }
        else { s.into() }
    }
}

pub struct OtherAssetItem {
    pub label: String,  // MID or Partial Asset Path
    pub asset: AssetItem,
    pub is_mess: bool,
    pub female: bool,

}
impl OtherAssetItem {
    pub fn new(label: impl AsRef<str>, asset: impl AsRef<str>, female: bool, flags: i32, is_mess: bool) -> Option<Self> {
        let label = label.as_ref().to_string();
        let lower = label.to_lowercase();
        let mut asset = AssetItem::new(asset, flags)?;
        if lower.contains("playable") && lower.starts_with("mpid") { asset.flags.insert(AssetItemFlags::Playable); }

        Some(Self{ label, female, is_mess, asset })
    }
    pub fn get_name(&self, labels: &AssetLabelTable, body_first: bool) -> &'static Il2CppString {
        if self.is_mess { self.asset.get_name(self.label.as_str()) }
        else {
            let s1 = labels.get_suffix_name(self.label.as_str());
            let s2 = labels.get_body(self.label.as_str()).map(|s| s.1.get());
            let a =
            if body_first { s2.or(s1).unwrap_or(self.label.as_str().into()) }
            else { s1.or(s2).unwrap_or(self.label.as_str().into()) }.to_string();
            self.asset.flags.modify_name(&a, self.asset.count)
        }
    }
}
pub struct AssetLabel {
    pub label: String,
    pub flag: AssetItemFlags,
    pub is_mess: bool,
}
impl AssetLabel {
    pub fn new(label: impl AsRef<str>, flags: i32) -> Self {
        let label = label.as_ref();
        let is_mess = (label.starts_with("M") && label.contains("ID_")) || label.starts_with("PCC");
        Self {
            label: label.to_string(), is_mess,
            flag: AssetItemFlags::from_bits(flags).unwrap_or(AssetItemFlags::empty()),
        }
    }
    pub fn get(&self) -> &'static Il2CppString {
        if self.is_mess { self.flag.modify_name(&Mess::get(self.label.as_str()).to_string(), 0)  }
        else { self.flag.modify_name(&self.label, 0) }
    }
}

#[derive(Clone)]
pub struct AssetItem {
    pub hash: i32,
    pub count: i32,
    pub kind: AssetType,
    pub flags: AssetItemFlags,
}


impl AssetItem {
    const PARSE_KINDS: [&str; 19] =
        [
            "BR_c", "CR_c", "DR_c", "ER_c", "FR_c", "DT_c", "CT_c",
            "uBody_", "uHead_", "uHair_", "uAcc_spine2_Hair", "uAcc_head_", "uAcc_spine", "uAcc_Eff", "uAcc_shield_",
            "Info", "Talk", "Demo", "Hub",
        ];
    pub fn new<'a>(asset_name: impl Into<&'a Il2CppString>, flags: i32) -> Option<Self> {
        let str = asset_name.into();
        let str2 = str.to_string();

        let kind =
            Self::PARSE_KINDS.iter().position(|x| str2.contains(x)).map(|pos| {
                match pos {
                    0..5 => AssetType::Mount(pos as u8),
                    5 => AssetType::Mount(2),
                    6 => AssetType::Mount(1),
                    7 => { AssetType::Body }
                    8 => { AssetType::Head }
                    9 | 10 => { AssetType::Hair }
                    11..15 => { AssetType::Acc((pos - 11) as u8) }
                    _ => { AssetType::AOC((pos - 15) as u8) }
                }
            })?;
        let hash = str.get_hash_code();
        let flags = AssetItemFlags::from_bits(flags)?;
        Some(Self { hash, count: 0, kind, flags, })
    }
    pub fn try_add_voice(voice: &str, voice_list: &mut Vec<String>, flag: i32) -> Option<Self> {
        if let Some(pos) = voice_list.iter().position(|x| *x == voice) {
            let voice = voice_list.remove(pos);
            let hash = crate::utils::hash_string(&voice);
            Some(AssetItem{ hash, count: 0, kind: AssetType::Voice, flags: AssetItemFlags::from_bits(flag).unwrap() })
        }
        else { None }
    }
    pub fn get_name(&self, mid: impl AsRef<str>) -> &'static Il2CppString {
        let mut s =
        if self.flags.contains(AssetItemFlags::AccessoryShop) { Mess::get(format!("MAID_{}", mid.as_ref())).to_string() }
        else { Mess::get(mid.as_ref()).to_string() };

        if s.len() < 1 { s = mid.as_ref().to_string(); }
        s = capitalize_first(s.as_str());
        self.flags.modify_name(&s, self.count)
    }
}
fn add_mess(string: &mut String, mess_id: impl AsRef<str>){
    let mess = Mess::get(mess_id.as_ref()).to_string();
    if mess.len() > 1 { add_str(string, mess); }
}
fn add_str(string: &mut String, value: impl AsRef<str>) {
    if !string.is_empty() { string.push(' '); }
    string.push_str(value.as_ref());
}

pub struct AssetGroup{
    pub label: &'static str,
    pub list: Vec<AssetItem>,
}
impl AssetGroup {
    pub fn new_job_group(line: &'static str, asset_list: &mut Vec<String>, hashes: &mut OutfitHashes, female: bool) -> Option<(Self, bool)> {
        let mut spilt = line.split_whitespace();
        let mut list = vec![];
        let label = spilt.next()?;
        let body = spilt.next()?;
        let mut has_body = false;
        if body.len() > 5 && !female {
            [Some(body), spilt.next()].iter().flatten().for_each(|x|{
                if let Some(a) = get_remove(asset_list, format!("uBody_{}", x).as_str()) {
                    if let Some(asset) = AssetItem::new(a.as_str(), 0){
                        hashes.add_ride_model(a.as_str());
                        list.push(asset);
                    }
                }
            });
        }
        else {
            let (b, f) = if female { (format!("uBody_{}F", body), 2) } else { (format!("uBody_{}M", body), 1) };
            asset_list.iter().filter(|x| x.contains(&b) )
                .for_each(|b|{
                    if let Some(asset) = AssetItem::new(b, f){
                        has_body = true;
                        hashes.add_body(b, female);
                        list.push(asset);
                    }
                });
            ACC.iter().enumerate().for_each(|(i, r)|{
                let flag2 = (1 << (i + 9)) | f;
                loop { if !try_search_and_add(&mut list, flag2, asset_list, hashes, "uAcc", Some(r), Some(b.as_str())) { break; } }
            });
            loop { if !try_search_and_add(&mut list, f|(1 << 17), asset_list, hashes, "uAcc_shield_", Some(b.as_str()), None) { break; } }
            if !female {
                for x in [("000", 0), ("707", 4)]{
                    if let Some(v) = get_remove(asset_list, format!("uBody_{}R_c{}", body, x.0).as_str()) {
                        if let Some(asset) = AssetItem::new(v.as_str(), x.1){
                            hashes.add_ride_model(v.as_str());
                            list.push(asset);
                        }
                    }
                }
                let mut count_aoc = 0;
                while let Some(suffix) = spilt.next() {
                    let aoc_female = count_aoc % 2 == 1;
                    if aoc_female == female {
                        if let Ok(num) = suffix.parse::<i32>() {
                            let flag = if num > 700 { 4 } else { 0 } + if female { 2 } else { 1 };
                            if let Some(asset) = get_remove(asset_list, format!("AOC_Info_{}", suffix).as_str()){
                                hashes.add_aoc(asset.as_str(), female);
                                if let Some(asset) = AssetItem::new(asset.as_str(), flag){ list.push(asset); }
                            }
                        }
                    }
                    count_aoc += 1;
                }
            }
        }
        if list.is_empty() { None } else { Some(Self { label, list }).zip(Some(has_body)) }
    }
    pub fn new_character_group(line: &'static str, asset_list: &mut Vec<String>, voice_list: &mut Vec<String>, hashes: &mut OutfitHashes) -> Option<Self> {
        let mut spilt = line.split_whitespace();
        let mut list = vec![];
        let label = spilt.next()?;
        if let Some((_, voice)) = label.split_once("_"){
            if let Some(voice_asset) = AssetItem::try_add_voice(voice, voice_list, 0) { list.push(voice_asset); }
            let fx_voice = voice.to_string() + "_E";
            if let Some(voice_asset) = AssetItem::try_add_voice(fx_voice.as_str(), voice_list, 4) {
                list.push(voice_asset);
            }
        }
        let mut current_suffix = String::new();
        let mut trimmed_suffix = String::new();
        let mut num = 0;
        let mut flag;
        let mut dark: Option<String> = None;
        while let Some(s) = spilt.next() {
            let (s1, f) =
                s.split_once(":")
                    .map(|(s, f)| (s, f.parse::<i32>().ok().unwrap_or(0)))
                    .unwrap_or((s, 0));

            if s1.starts_with("c") {
                current_suffix = s1.to_string();
                trimmed_suffix = s1.trim_start_matches("c").to_string();
                num = trimmed_suffix.parse::<i32>().unwrap_or(0);
                if (num >= 510 && num < 517) || (num >= 530 && num < 538) || (num >= 560 && num < 567) || (num >= 580 && num < 588) {
                    dark = Some((num + 7).to_string());
                }
                flag = f;
                try_search_and_add(&mut list, f, asset_list, hashes, "uHead", None, Some(current_suffix.as_str()));
                if !line.contains("uHair") { try_search_and_add(&mut list, f, asset_list, hashes, "uHair", None, Some(current_suffix.replace("c", "h").as_str())); }
                try_search_and_add(&mut list, f, asset_list, hashes, "AOC_Info_", None, Some(current_suffix.as_str()));
                try_search_and_add(&mut list, f | 32, asset_list, hashes, "AOC_Info", None, Some(format!("{}_Eng", current_suffix).as_str()));
                if !line.contains("Hair") {
                    try_search_and_add(&mut list, f, asset_list, hashes, "uAcc_spine2_Hair", None, Some(trimmed_suffix.as_str()));
                    for x in [("e", 32), ("h", 16), ("k", 256)]{
                        let suffix = format!("{}{}", trimmed_suffix, x.0);
                        try_search_and_add(&mut list, f|x.1, asset_list, hashes, "uAcc_spine2_Hair", None, Some(suffix.as_str()));
                        try_search_and_add(&mut list, f|x.1, asset_list, hashes, "uHair_h", None, Some(suffix.as_str()));
                    }
                }
                try_search_and_add(&mut list, f, asset_list, hashes, "AOC_Demo_", None, Some(current_suffix.as_str()));
                if !line.contains("Talk)") { try_search_and_add(&mut list, f, asset_list, hashes, "AOC_Talk_", None, Some(current_suffix.as_str())); }
                if !line.contains("Hub_") { try_search_and_add(&mut list, f, asset_list, hashes, "AOC_Hub_", None, Some(current_suffix.as_str())); }
                ACC.iter().enumerate().for_each(|(i, x)| {
                    let flag2 = flag | (1 << (i + 9));
                    let acc = format!("{}{}", x, trimmed_suffix);
                    try_search_and_add(&mut list, flag2, asset_list, hashes, "uAcc", Some("head"), Some(acc.as_str()));
                    try_search_and_add(&mut list, flag2, asset_list, hashes, "uAcc", Some("spine2"), Some(acc.as_str()));
                    let acc_h = format!("{}{}h", x, trimmed_suffix);
                    try_search_and_add(&mut list, flag2|16, asset_list, hashes, "uAcc", Some("head"), Some(acc_h.as_str()));
                    try_search_and_add(&mut list, flag2|16, asset_list, hashes, "uAcc", Some("spine2"), Some(acc_h.as_str()));
                });
                try_search_and_add(&mut list, flag|(1 << 17), asset_list, hashes, "uAcc_shield_", None, Some(trimmed_suffix.as_str()));
                if trimmed_suffix.ends_with("b") {
                    for r in [("h", 16), ("c", 0)] {
                        let hub = trimmed_suffix.replace("b", r.0);
                        ACC.iter().enumerate().for_each(|(i, x)| {
                            let flag2 = flag | (1 << (i + 9)) | r.1;
                            let acc = format!("{}{}", x, hub);
                            try_search_and_add(&mut list, flag2, asset_list, hashes, "uAcc_", None, Some(acc.as_str()));
                        });
                    }
                }
                if let Some(dark) = dark.as_ref() {
                    try_search_and_add(&mut list, f|4, asset_list, hashes, "uHead", None, Some(dark.as_str()));
                    try_search_and_add(&mut list, f|4, asset_list, hashes, "uHair", None, Some(dark.as_str()));
                    try_search_and_add(&mut list, f|4, asset_list, hashes, "uAcc_spine2_Hair", None, Some(dark.as_str()));
                    ACC.iter().enumerate().for_each(|(i, x)| {
                        let flag2 = flag | (1 << (i + 9));
                        try_search_and_add(&mut list, flag2, asset_list, hashes, "uAcc", Some(x), Some(dark.as_str()));
                    });
                }
            }
            else if current_suffix.is_empty() { continue; }
            else {
                if s1 == "w" { try_search_and_add(&mut list, f | 16, asset_list, hashes, "uBody", Some("Wear"), Some(current_suffix.as_str())); }
                else if s1.starts_with("Wear") { try_search_and_add(&mut list, f | 16, asset_list, hashes, "uBody", Some(s1), None); }
                else if s1 == "h" {
                    let flag =
                        if s1.ends_with("e") { 32 } else if s1.ends_with("h") { 16 } else if s1.ends_with("k") { 256 } else { 0 };

                    let hair = format!("Hair{}", trimmed_suffix);
                    try_search_and_add(&mut list, f | flag, asset_list, hashes, "uAcc_spine2", None, Some(hair.as_str()));
                    if dark.is_some() {
                        let hair = format!("Hair{}", num + 7);
                        try_search_and_add(&mut list, f | flag, asset_list, hashes, "uAcc_spine2", None, Some(hair.as_str()));
                    }
                }
                else if let Some(voice_asset) = AssetItem::try_add_voice(s1, voice_list, f) { list.push(voice_asset); }
                else if try_search_and_add(&mut list, f, asset_list, hashes, "uBody", Some(s1), Some(current_suffix.as_str())) {
                    if let Some(dark) = dark.as_ref() {
                        try_search_and_add(&mut list, f | 64, asset_list, hashes, "uBody", Some(s1), Some("c000"));
                        try_search_and_add(&mut list, f | 4, asset_list, hashes, "uBody", Some(s1), Some(dark.as_str()));
                    }
                    if s1.contains("0A") || s1.contains("0B") || s1.contains("0D") || s1.contains("0E") {
                        try_search_and_add(&mut list, f|256, asset_list, hashes, "uBody", Some(s1.replace("0", "1").as_str()), Some(current_suffix.as_str()));
                    }
                } else { try_search_and_add(&mut list, f, asset_list, hashes, "", Some(s1), None); }
            }
        }
        if list.is_empty() { None }
        else { Some(Self { label, list }) }
    }
    pub fn new_aid_group(line: &'static str, asset_list: &mut Vec<String>, hashes: &mut OutfitHashes) -> Option<Self> {
        let mut spilt = line.split_whitespace();
        let mut list = vec![];
        let label = spilt.next()?;
        let body = spilt.next()?;
        let flag = 1 << 30;
        for x in [("M", 1), ("F", 2)]{
            let search = format!("{}{}", body, x.0);
            loop {
                if !try_search_and_add(&mut list, x.1|flag, asset_list, hashes, "uAcc", Some(search.as_str()), None) { break; }
            }
            loop {
                if !try_search_and_add(&mut list, x.1|flag, asset_list, hashes, "uBody", Some(search.as_str()), None) { break; }
            }
        }
        loop {
            if !try_search_and_add(&mut list, flag, asset_list, hashes, "uAcc", Some(body), None) { break; }
        }
        loop {
            if !try_search_and_add(&mut list, flag, asset_list, hashes, "uAcc", Some(body), None) { break; }
        }
        if list.is_empty() { None } else { Some(Self { label, list }) }
    }
}

fn try_search_and_add(
    list: &mut Vec<AssetItem>,
    flag: i32,
    asset_list:  &mut Vec<String>,
    hashes: &mut OutfitHashes,
    prefix: &str, body: Option<&str>, suffix: Option<&str>) -> bool {
    if let Some(pos) = asset_list.iter().position(|x| x.starts_with(prefix) && body.is_none_or(|b| x.contains(b)) && suffix.is_none_or(|s| x.ends_with(s))){
        let str = asset_list.remove(pos);
        let morph = str.contains("uHead") && (str.contains("702") || str.contains("703"));
        if let Some(mut asset) = AssetItem::new(str.clone(), flag) {
            hashes.add(&str, asset.kind);
            asset.count = list.iter().filter(|x| x.flags == asset.flags && x.kind == asset.kind).count() as i32;
            if morph { asset.flags.insert(AssetItemFlags::NoPhotograph); }
            list.push(asset);
            return true;
        }
    }
    false
}
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