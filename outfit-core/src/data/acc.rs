use crate::localize::MenuTextCommand;
use super::*;
#[derive(Default)]
pub struct AccEntry {
    pub hash: i32,
    pub label: Option<String>,
    pub acc_obj: String,
    pub body: Option<String>,
    pub acc_suffix: Option<String>,
    pub flag: i32,
    pub count: i32,
}
impl AccEntry {
    pub fn new(hash: i32, label: Option<&str>, acc_asset: &str, body_suffix: Option<&str>, flags: i32, count: i32) -> Self {
        let mut asset_str = acc_asset.to_string();
        let mut flag = flags;
        if flag & 128 != 0 && label.is_some() {
            Self { hash, label: label.map(|v| v.to_string()), flag, acc_obj: String::new(), body: None, acc_suffix: None, count };
        }
        if acc_asset.contains("Mrp") {
            flag |= 8;
            asset_str = asset_str.replace("Mrp", "");
        }
        let ending = asset_str.split("_").last().unwrap();
        let mut acc_obj = ending.to_string();

        let body = body_suffix.map(|s| s.to_string());
        let mut acc_suffix = None;
        if let Some(label) = label {
            if label.starts_with("MPID_") || label.starts_with("MJID_") { flag |= 32; }
            else if label.starts_with("MAID_") { flag |= 64; }
        }
        let label = label.map(|l| l.to_string());
        if let Some(bodys) = body_suffix {
            let mut spilt = ending.split(bodys);
            if let Some(sp) = spilt.next() {
                acc_obj = sp.to_string();
            }
            if let Some(suffix) = spilt.next() {
                acc_suffix = None;
                if suffix == "h" { flag |= 1; }
                else if suffix == "e" { flag |= 2; }
                else if suffix == "k" { flag |= 4; }
                else if suffix.len() > 0 { acc_suffix = Some(suffix.to_string()); }
            }
        }
        if flag & 64 != 0 || body_suffix.is_none() {
            if label.is_some() {
                let len = asset_str.len();
                if let Some(_pos) = acc_asset.rfind("M").filter(|x| *x > (len - 3)){
                    acc_obj = String::new();
                    acc_suffix = acc_asset.split("M").last().map(|v| format!("M{}", v));
                }
                else if let Some(_pos) = acc_asset.rfind("F").filter(|x| *x > (len - 3)){
                    acc_obj = String::new();
                    acc_suffix = acc_asset.split("F").last().map(|v| format!("F{}", v));

                }
            }
        }
        Self { hash, label, flag, acc_obj, body, acc_suffix, count }
    }
    pub fn get_name(&self, outfit_data: &OutfitData) -> &'static Il2CppString {

        if self.flag & 128 != 0 && self.label.is_some() {
            let label = self.label.as_ref().unwrap();
            return
            if self.flag & 16 != 0 { format!("{}: {}", MenuTextCommand::Engage.get(), Mess::get(label)).into() }
            else { Mess::get(label) }
        }
        if self.flag & 64 != 0 && self.label.is_some() {
            let count = if self.count > 0 { format!(" {}", self.count+1) } else { "".to_string() };
            return
                if let Some(acc) = self.acc_suffix.as_ref() {
                    format!("{} {}{}", Mess::get(self.label.as_ref().unwrap()), acc, count).into()
                }
            else {
                format!("{}{}", Mess::get(self.label.as_ref().unwrap()), count).into()
            };
        }
        let mut out =
            if let Some(label) = self.label.as_ref(){
                format!("{} {}", Mess::get(label), self.acc_obj) }
            else if let Some((body_label, _)) = self.body.as_ref().and_then(|b| outfit_data.try_get_asset_label(&b)){
                format!("{} {}", self.acc_obj, body_label) }
            else { self.acc_obj.clone() };

        if let Some(v) = self.acc_suffix.as_ref() {
            out.push_str(" ");
            out.push_str(v.as_str());
        }
        else if self.flag & 1 != 0 {
            out.push_str(" ");
            out.push_str(Mess::get("MID_Hub_Solanel").to_string().as_str());
        }
        else if self.flag & 2 != 0 {
            out.push_str(" ");
            out.push_str(MenuTextCommand::Engage.to_string().as_str());
        }
        else if self.flag & 4 != 0 { out.push_str(" Kings"); }
        if self.flag & 8 != 0 {
            Mess::set_argument(0, out);
            return Mess::get("MPID_Morph_Prefix");
        }
        out.into()
    }
}