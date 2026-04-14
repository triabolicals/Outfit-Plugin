use std::collections::{HashMap, HashSet};
use unity::prelude::Il2CppString;
use crate::{hash_string, AssetType};

#[derive(Default)]
pub struct OutfitHashes {
    pub male_ou: Vec<(i32, i32)>,
    pub female_ou: Vec<(i32, i32)>,
    pub male_u: Vec<i32>,
    pub female_u: Vec<i32>,
    pub head_hair: HashMap<i32, i32>,
    pub oacc_pair: HashMap<i32, i32>,
    pub acc_head: HashSet<i32>,
    pub acc_spine: HashSet<i32>,
    pub acc_shield: HashSet<i32>,
    pub acc_eff: HashSet<i32>,
    pub mount_ou: Vec<(i32, i32)>,
    pub body: HashMap<i32, String>,
    pub head: HashMap<i32, String>,
    pub hair: HashMap<i32, String>,
    pub o_hair: HashMap<i32, String>,
    pub o_body: HashMap<i32, String>,
    pub o_acc: HashMap<i32, String>,
    pub aoc: HashMap<i32, String>,
    pub aoc_m: HashSet<i32>,
    pub aoc_f: HashSet<i32>,
    pub acc: HashMap<i32, String>,
    pub mounts: HashMap<i32, String>,
    pub voice: HashMap<i32, String>,
    pub rigs: HashMap<i32, String>,
}
impl OutfitHashes {
    pub fn new() -> Self {
        let mut new: OutfitHashes = Default::default();
        new.add_acc("uAcc_head_null", Some(1));
        for i in 1..4 { new.add_acc("null", Some(i)); }
        new.add_hair("uHair_null", None);
        new.add_head("uHead_null");
        new.add_body("uBody_null", false);
        new.add_body("uBody_null", true);
        new
    }
    pub fn add(&mut self, asset: &String, ty: AssetType) {
        match ty {
            AssetType::Head => { self.add_head(asset); }
            AssetType::Hair => { self.add_hair(asset, None); }
            AssetType::Body => {
                let female = ["F_c", "f_c", "F1_", "F2_", "F3_", "F4_"].iter().any(|a| asset.contains(*a));
                self.add_body(asset, female);
            }
            AssetType::Mount(_) => { self.add_ride_model(asset); }
            AssetType::Acc(kind) => { self.add_acc(asset, Some(kind as i32)); }
            AssetType::AOC(_) => {
                let hashcode = hash_string(asset);
                self.aoc.insert(hashcode, asset.to_string());
            }
            _ => {}
        }
    }
    pub fn get_body_hash(&self, body: impl Into<&'static Il2CppString>) -> Option<i32> {
        let hash = body.into();
        let hash_code = hash.get_hash_code();
        if self.body.contains_key(&hash_code) { Some(hash_code) }
        else if self.o_body.contains_key(&hash_code) { Some(hash_code) }
        else { None }
    }
    pub fn get_ohair(&self, head_hair_hash: i32) -> Option<&'static Il2CppString> {
        self.head_hair.get(&head_hair_hash).and_then(|hash| self.o_hair.get(hash)).map(|v| v.into())
    }
    pub fn get_obody(&self, ubody_hash: i32) -> Option<&'static Il2CppString> {
        self.male_ou.iter().find(|x| x.0 == ubody_hash)
            .or_else(|| self.female_ou.iter().find(|x| x.0 == ubody_hash))
            .map(|x| x.1)
            .and_then(|x| self.o_body.get(&x))
            .map(|x| x.into())
    }
    pub fn get_mount_obody(&self, mount_hash: i32) -> Option<&'static Il2CppString> {
        self.mount_ou.iter()
            .find(|x| x.0 == mount_hash)
            .map(|x| x.1)
            .and_then(|x| self.o_body.get(&x))
            .map(|x| x.into())
    }
    pub fn get_oacc(&self, uacc_hash: i32) -> Option<&'static Il2CppString> { self.oacc_pair.get(&uacc_hash).and_then(|x| self.o_acc.get(x)).map(|x| x.into()) }
    pub fn add_acc(&mut self, asset: impl Into<&'static Il2CppString>, acc_kind: Option<i32>) -> i32 {
        let asset = asset.into();
        let hashcode = asset.get_hash_code();
        let str = asset.to_string();
        let ostr = str.trim_start_matches("uAcc");
        if let Some(oacc) = self.o_acc.iter().find(|v| v.1.contains(ostr)) { self.oacc_pair.insert(hashcode, *oacc.0); }
        self.acc.insert(hashcode, str);

        if let Some(kind) = acc_kind {
            match kind {
                0 => { self.acc_head.insert(hashcode); }
                1 => { self.acc_spine.insert(hashcode); }
                2 => { self.acc_shield.insert(hashcode); }
                3 => { self.acc_eff.insert(hashcode); }
                _ => {}
            }
        }
        else {
            if asset.str_contains("uAcc_head_") { self.acc_head.insert(hashcode); }
            else if asset.str_contains("uAcc_spine2_") { self.acc_spine.insert(hashcode); }
            else if asset.str_contains("uAcc_shield_") { self.acc_shield.insert(hashcode); }
        }
        hashcode
    }
    pub fn add_body(&mut self, asset: impl Into<&'static Il2CppString>, is_female: bool) -> i32 {
        let asset = asset.into();
        let hashcode = asset.get_hash_code();
        if self.body.contains_key(&hashcode) { return hashcode }
        let str = asset.to_string();
        self.body.insert(hashcode, str.clone());
        let spilt = str.split("_").collect::<Vec<&str>>();
        if spilt.len() > 2 {
            let obody = format!("oBody_{}_{}", spilt[1], spilt[2]);
            if let Some(obody_hash) = self.o_body.iter().find(|x| *x.1 == obody)
                .or_else(|| self.o_body.iter().find(|x| x.1.contains(spilt[1]) && x.1.contains("c000")))
                .or_else(|| self.o_body.iter().find(|x| x.1.contains(spilt[1])))
                .map(|x| *x.0)
            {
                if is_female { self.female_ou.push((hashcode, obody_hash)); }
                else { self.male_ou.push((hashcode, obody_hash)); }
            }
            else if spilt[1].chars().position(|c| c.is_numeric()).is_some_and(|x| x != 3) {
                let sub = &spilt[1][0..4];
                let gender = if is_female { "F_c" } else { "M_c" };
                if let Some(obody_hash) = self.o_body.iter().find(|x| x.1.contains(sub) && x.1.contains(gender)).map(|x| *x.0) {
                    if is_female { self.female_ou.push((hashcode, obody_hash)); }
                    else { self.male_ou.push((hashcode, obody_hash)); }
                }
            }
            if is_female { self.female_u.push(hashcode); }
            else { self.male_u.push(hashcode); }
        }
        hashcode
    }
    pub fn add_head(&mut self, asset: impl Into<&'static Il2CppString>) -> i32 {
        let asset = asset.into();
        let hashcode = asset.get_hash_code();
        let str = asset.to_string();
        self.head.insert(hashcode, str.clone());
        let spilt = str.split("_c").collect::<Vec<&str>>();
        if spilt.len() > 1 {
            let ohair = format!("oHair_h{}", spilt[1]);
            if let Some(ohair_hash) = self.o_hair.iter().find(|x| *x.1 == ohair)
                .or_else(|| self.o_hair.iter().find(|x| x.1.contains(spilt[1])))
                .map(|x| *x.0){
                self.head_hair.insert( hashcode, ohair_hash);
            }
        }
        hashcode
    }
    pub fn add_hair(&mut self, asset: impl Into<&'static Il2CppString>, head: Option<&str>) -> i32 {
        let asset = asset.into();
        let hashcode = asset.get_hash_code();
        let str = asset.to_string();
        self.hair.insert(hashcode, str.clone());
        if !self.head_hair.contains_key(&hashcode) {
            if let Some(suffix) = head {
                let ohair = format!("oHair_h{}", suffix);
                if let Some(ohair_hash) = self.o_hair.iter().find(|x|*x.1 == ohair)
                    .or_else(|| self.o_hair.iter().find(|x| x.1.contains(suffix)))
                    .map(|x| *x.0)
                {
                    self.head_hair.insert( hashcode, ohair_hash);
                    return hashcode;
                }
            }
            let spilt_by = if str.contains("_Hair") { "_Hair" } else { "_h" };
            if let Some(suffix) = head.or_else(||str.split(spilt_by).last()) {
                let mut sfx = suffix.to_string();
                while sfx.len() >= 3 {
                    let ohair = format!("oHair_h{}", sfx);
                    if let Some(ohair_hash) = self.o_hair.iter().find(|x|*x.1 == ohair)
                        .or_else(|| self.o_hair.iter().find(|x| x.1.contains(sfx.as_str())))
                        .map(|x| *x.0)
                    {
                        self.head_hair.insert( hashcode, ohair_hash);
                        break;
                    }
                    else { sfx.pop(); }
                }
            }
        }
        hashcode
    }
    pub fn add_aoc(&mut self, asset: impl Into<&'static Il2CppString>, female: bool) -> i32 {
        let asset = asset.into();
        let hashcode = asset.get_hash_code();
        self.aoc.insert(hashcode, asset.to_string());
        if female { self.aoc_f.insert(hashcode); } else { self.aoc_m.insert(hashcode); }
        hashcode
    }
    pub fn add_ride_model(&mut self, asset: impl Into<&'static Il2CppString>) -> i32 {
        let asset = asset.into();
        let hashcode = asset.get_hash_code();
        let str = asset.to_string();
        self.mounts.insert(hashcode, str.clone());
        let spilt = str.split("_").collect::<Vec<&str>>();
        if spilt.len() > 2 {
            let obody = format!("oBody_{}_{}", spilt[1], spilt[2]);
            if let Some(obody_hash) = self.o_body.iter().find(|x| *x.1 == obody)
                .or_else(|| self.o_body.iter().find(|x| x.1.contains(spilt[1])))
                .map(|x| *x.0)
            {
                self.mount_ou.push((hashcode, obody_hash));
            }
        }
        hashcode
    }
    pub fn get_engaged_hair(&self, model: &Il2CppString) -> Option<&'static Il2CppString> {
        let mut str = model.to_string();
        if str.contains("oHair_") && !str.ends_with("e") {
            if str.chars().last().is_some_and(|c| !c.is_numeric()) { str.pop(); }
            self.o_hair.iter().find(|x| x.1.contains(str.as_str()) && x.1.ends_with("e")).map(|x| Il2CppString::new(x.1.as_str()))
        }
        else if !str.ends_with("e") {
            if str.chars().last().is_some_and(|c| !c.is_numeric()) { str.pop(); }
            self.hair.iter().find(|x| x.1.contains(str.as_str()) && x.1.ends_with("e") ).map(|x| Il2CppString::new(x.1.as_str()))
        }
        else { None }
    }
}
