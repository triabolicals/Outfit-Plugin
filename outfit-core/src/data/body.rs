pub use super::*;

#[derive(Default)]
pub struct GenderAssetList {
    pub male: Vec<OutfitEntry>,
    pub female: Vec<OutfitEntry>,
}
impl GenderAssetList {
    pub fn add(&mut self, hash: i32, female: bool, mid: impl AsRef<str>, flag: i32) {
        let string = mid.as_ref().to_string();
        let mut entry = OutfitEntry::new_with_flags(hash, mid, flag);
        if female {
            entry.label.count = self.female.iter().filter(|e| e.label.name == string && flag & 32 != 0).count() as i32;
            self.female.push(entry);
        }
        else {
            entry.label.count = self.male.iter().filter(|e| e.label.name == string && flag & 32 != 0).count() as i32;
            self.male.push(entry);
        }
    }
}

pub struct ClassBodyEntry { pub hash: i32, pub flags: i32, }

#[derive(Default)]
pub struct ClassBodyList {
    pub class_label: String,
    pub body_label: String,
    pub list: Vec<ClassBodyEntry>,
}
impl ClassBodyList {
    pub fn new(label: impl AsRef<str>, flag: i32, body_label: impl AsRef<str>) -> Self {
        let label =
            if flag & 32 == 0 { format!("MJID_{}", label.as_ref()) }
            else { label.as_ref().to_string() };
        Self {
            class_label: label,
            list: vec![],
            body_label: body_label.as_ref().to_string(),
        }
    }
}