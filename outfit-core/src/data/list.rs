use std::collections::HashMap;
use std::io::{Cursor, Read};
use engage::mess::Mess;
use unity::{prelude::Il2CppString, system::List};
use crate::{Asset, AssetColor, AssetType, ColorPreset, CustomAssetMenuItem, OutfitHashes, UnitAssetMenuData};
use crate::data::item::*;
use crate::data::util::parse_label;

pub struct OutfitLists {
    pub null: AssetGroup,   // 1st
    pub other: Vec<OtherAssetItem>, // 3rd
    pub engaged: Vec<OtherAssetItem>,
    pub job_m: Vec<AssetGroup>,
    pub job_f: Vec<AssetGroup>,
    pub char_m: Vec<AssetGroup>,    // First
    pub char_f: Vec<AssetGroup>,    // Second
    pub aids: Vec<AssetGroup>,  // 2nd
    pub color_presets: Vec<ColorPreset>,
    pub job_count: (i32, i32),
    pub skin: HashMap<i32, AssetColor>,
}

impl OutfitLists {
    pub fn new() -> Self {
        let mut null = AssetGroup{
            label: "MID_SYS_None",
            list: ["uBody_null", "uHead_null", "uHair_null", "uAcc_head_null"].iter().flat_map(|x| AssetItem::new(x, 0)).collect(),
        };
        null.list[1].flags.insert(AssetItemFlags::NoPhotograph);
        let null_hash = crate::utils::hash_string("null");
        for x in 1..4 {
            let acc_null =
            AssetItem{ hash: null_hash, count: 0, kind: AssetType::Acc(x), flags: AssetItemFlags::empty(), };
            null.list.push(acc_null);
        }
        let data = include_bytes!("../../data/color.bin");
        let size = data.len() as u64;
        let mut color = Cursor::new(data);
        let mut buff: [u8; 26] = [0; 26];
        let mut color_presets = vec![];
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
                color_presets.push(ColorPreset{ colors, engaged, count, label, });
            }
        }
        Self {
            null, color_presets,
            job_count: (0, 0),
            other: vec![], engaged: vec![], job_m: vec![], job_f: vec![], char_m: vec![], char_f: vec![], aids: vec![],
            skin: HashMap::new(),
        }
    }
    pub fn add_other_body(&mut self, mid: impl AsRef<str>, asset: impl AsRef<str>, female: bool, flag: i32, is_mess: bool) {
        let str = mid.as_ref().to_string();
        if let Some(mut body) = OtherAssetItem::new(mid, asset, female, flag, is_mess) {
            body.asset.count = self.other.iter().filter(|x| x.asset.flags == body.asset.flags && x.label == str).count() as i32;
            self.other.push(body);
        }
    }
    pub fn add_engaged_body(&mut self, mid: impl AsRef<str>, asset: impl AsRef<str>, female: bool) {
        if let Some(body) = OtherAssetItem::new(mid, asset, female, 0, true) {
            self.engaged.push(body);
        }
    }
    pub fn add_other_to_list(&mut self, mut asset: OtherAssetItem) {
    let str = asset.label.as_ref();
        asset.asset.count = self.other.iter()
            .filter(|x| x.asset.flags == asset.asset.flags && x.label == str && x.asset.kind == asset.asset.kind)
            .count() as i32;
        self.other.push(asset);
    }
    pub fn add<C>(&mut self, asset: impl AsRef<str>, female: bool, mid: Option<C>, flag: i32)
    where C: AsRef<str> + ToString
    {
        let is_mess = mid.is_some();
        let name = mid.map(|m| m.as_ref().to_string()).unwrap_or({
            let mut a = asset.as_ref().to_string();
            if a.contains("_") { a = a.split_once("_").unwrap().1.to_string(); }
            if a.starts_with("head_") || a.starts_with("spine") || a.starts_with("shield_") || a.starts_with("Eff_"){
                a = a.split_once("_").unwrap().1.to_string();
            }
            a
        });
        if let Some(asset) = OtherAssetItem::new(name, asset, female, flag, is_mess) {
            self.add_other_to_list(asset);
        }
    }
    pub fn final_add(&mut self, hashes: &mut OutfitHashes) {
        let sola_hash = hashes.add_acc("uBody_Msc0AT_c000", Some(3));
        self.other.push(
            OtherAssetItem{
                label: "MPID_Sola".to_string(), is_mess: true,female: false,
                asset: AssetItem { hash: sola_hash, count: 0, kind: AssetType::Acc(3), flags: AssetItemFlags::empty(), },
            }
        );
        [("uBody_DummyM", false), ("uBody_DummyF", true)].iter().for_each(|i|{
            if let Some(asset) = OtherAssetItem::new("Dummy", i.0, i.1, 0, false) {
                hashes.body.insert(asset.asset.hash, i.0.to_string());
                if i.1 { hashes.female_u.push(asset.asset.hash); }
                else { hashes.male_u.push(asset.asset.hash); }
                self.other.push(asset);
            }
        });
        ["AM_c000", "AM_c998", "AM_c999", "BR_c000"].iter().enumerate()
            .for_each(|(i, x)|{
                let asset = format!("uBody_Box0{}", x);
                if i == 3 { hashes.add_ride_model(&asset); }
                else {
                    let hash = crate::utils::hash_string(&asset);
                    hashes.body.insert(hash, asset.clone());
                }
                self.add(asset, false, Some("Box"), 0);
            });
        for x in 1..3 {
            let head = format!("uHead_dummy{}", x);
            if let Some(mut asset) = OtherAssetItem::new("Dummy", head.as_str(), false, 0, false) {
                hashes.head.insert(asset.asset.hash, head);
                asset.asset.flags.insert(AssetItemFlags::NoPhotograph);
                self.other.push(asset);
            }
        }
    }
    pub fn add_menu_items(&self, kind: AssetType, female: bool, char: bool, other: bool, labels: &AssetLabelTable, menu_item_list: &mut List<CustomAssetMenuItem>) {
        let photo = UnitAssetMenuData::is_photo_graph();
        let check_photo_flag = (kind == AssetType::Head) == photo;
        let mut acc_kind = None;
        let mut gender_restrict = false;
        let mut kind2 = kind;
        match kind {
            AssetType::AOC(_)|AssetType::Body => { gender_restrict = true; }
            AssetType::Acc(k) => {
                acc_kind = Some(k);
                kind2 = if k > 0 { AssetType::Acc(k - 1) } else { AssetType::Acc(0) };
            }
            AssetType::Mount(_) => {
                self.job_m.iter().for_each(|s| {
                    s.list.iter().filter(|x| x.kind == kind)
                        .for_each(|h| { menu_item_list.add(CustomAssetMenuItem::new_asset2(&h, s.label)); });
                });
            }
            _ => {}
        };
        self.null.list.iter().filter(|v| v.kind == kind2 ).for_each(|v|{
            let item = CustomAssetMenuItem::new_asset2(&v, self.null.label);
            item.name = Mess::get("MID_SYS_None");
            menu_item_list.add(item);
        });
        if char {
            if gender_restrict {
                if female { &self.char_f } else { &self.char_m }
                    .iter()
                    .for_each(|char| {
                        char.list.iter()
                            .filter(|a| a.kind == kind2 && (!check_photo_flag || (check_photo_flag != a.flags.contains(AssetItemFlags::NoPhotograph))))
                            .for_each(|h| { menu_item_list.add(CustomAssetMenuItem::new_asset2(&h, char.label)); });
                    });
            }
            else {
                if female && acc_kind.is_none() {
                    self.char_f.iter().chain(self.char_m.iter())
                        .for_each(|char| {
                            char.list.iter()
                                .filter(|a| a.kind == kind2 && (!check_photo_flag || (check_photo_flag != a.flags.contains(AssetItemFlags::NoPhotograph))))
                                .for_each(|h| { menu_item_list.add(CustomAssetMenuItem::new_asset2(&h, char.label)); });
                        });
                } else {
                    self.char_m.iter().chain(self.char_f.iter())
                        .for_each(|char| {
                            char.list.iter()
                                .filter(|a| a.kind == kind2 && (!check_photo_flag || (check_photo_flag != a.flags.contains(AssetItemFlags::NoPhotograph))))
                                .for_each(|h| { menu_item_list.add(CustomAssetMenuItem::new_asset2(&h, char.label)); });
                        });
                }
            }
        }
        if other {
            let gender = if female { AssetItemFlags::Female } else { AssetItemFlags::Male };
            self.aids.iter()
                .for_each(|char| {
                    char.list.iter()
                        .filter(|a|
                            a.kind == kind2 && (!check_photo_flag || (check_photo_flag != a.flags.contains(AssetItemFlags::NoPhotograph)))
                            && (gender_restrict && a.flags.contains(gender) || !gender_restrict)
                        )
                        .for_each(|h| { menu_item_list.add(CustomAssetMenuItem::new_asset2(&h, char.label)); });
                });

            self.other.iter()
                .filter(|x|{
                    ((gender_restrict && (x.female == female)) || !gender_restrict) &&
                    x.asset.kind == kind2 && (!check_photo_flag || (check_photo_flag != x.asset.flags.contains(AssetItemFlags::NoPhotograph)))
                })
                .for_each(|a|{ menu_item_list.add(CustomAssetMenuItem::new_asset3(&a, labels, false)); });
        }
        if let Some(acc) = acc_kind {
            let menu_kind = AssetType::Acc(acc);
            menu_item_list.iter_mut().for_each(|a|{ a.menu_kind = Asset(menu_kind); });
        }
    }
}

pub struct AssetLabelTable {
    pub body: HashMap<String, AssetLabel>,
    pub suffix: HashMap<String, AssetLabel>,
}
impl AssetLabelTable {
    pub fn new() -> Self {
        let mut section = 0;
        let mut body = HashMap::new();
        let mut suffix = HashMap::new();
        include_str!("../../data/labels2.txt").lines()
            .for_each(|line|{
                if line.starts_with("END") { section += 1; }
                else if section < 4 {
                    let mut line = line.split_whitespace();
                    if let Some((name, value)) = line.next().zip(line.next()) {
                        let (label, flag) = parse_label(value);
                        if section == 0 { body.insert(label, AssetLabel::new(name, flag)); }
                        else {
                            suffix.insert(label, AssetLabel::new(name, flag));
                            while let Some(s) = line.next() {
                                if s.starts_with("c") {
                                    let (label, flag) = parse_label(s);
                                    suffix.insert(label, AssetLabel::new(name, flag));
                                }
                            }
                        }
                    }
                }
            });
        Self { body, suffix }
    }
    pub fn get_suffix(&self, asset: &str) -> Option<(&String, &AssetLabel)>{
        self.suffix.iter().find(|s| asset.ends_with(s.0))
            .or_else(||{
                let mut a = asset.to_string();
                a.pop();
                self.suffix.iter().find(|s| a.ends_with(s.0.as_str()))
            })
    }
    pub fn get_body(&self, asset: &str) -> Option<(&String, &AssetLabel)> {
        self.body.iter().find(|s| asset.contains(s.0))
    }
    pub fn get_suffix_name(&self, asset: &str) -> Option<&'static Il2CppString> {
        self.get_suffix(asset).map(|x|{
            let out = x.1.get();
            if asset.ends_with(x.0) { out }
            else { format!("{} {}", out, asset.split(x.0.as_str()).last().unwrap_or(" ")).into() }
        })
    }
}