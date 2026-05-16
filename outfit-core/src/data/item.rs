use bitflags::bitflags;
use engage::{gamedata::assettable::*, gamedata::Gamedata, gamedata::item::ItemData, mess::Mess};
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
        const LabelFemale = 1 << 21;
        const LabelMess = 1 << 22;
        const MPID = 1 << 23;
        const MJID = 1 << 24;
        const MAID = 1 << 25;
        const DEMO = 1 << 26;
        const HUB = 1 << 27;
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
    pub fn get_name(&self, mid: impl AsRef<str>) -> &'static Il2CppString {
        let label =
            if self.flags.contains(AssetItemFlags::AccessoryShop) || self.flags.contains(AssetItemFlags::MAID) { format!("MAID_{}", mid.as_ref()) }
            else if self.flags.contains(AssetItemFlags::MPID) { format!("MPID_{}", mid.as_ref()) }
            else if self.flags.contains(AssetItemFlags::MJID) { format!("MJID_{}", mid.as_ref()) }
            else if self.flags.contains(AssetItemFlags::HUB) { format!("Hub{}", mid.as_ref()) }
            else if self.flags.contains(AssetItemFlags::DEMO) { format!("Demo_{}", mid.as_ref()) }
            else { mid.as_ref().to_string() };
        println!("Other Asset Label: {}", label);
        let mut s = Mess::get(label).to_string();

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