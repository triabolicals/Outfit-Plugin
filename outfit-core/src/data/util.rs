use engage::{gamedata::{Gamedata, JobData}, gamedata::assettable::*,};
#[derive(Default)]
pub struct AssetTableIndexes {
    pub mode_1: Vec<i32>,
    pub mode_2: Vec<i32>,
}
impl AssetTableIndexes {
    pub fn add(&mut self, entry: &AssetTable) {
        if entry.mode == 1 || entry.mode == 0 { self.mode_1.push(entry.parent.index); }
        if entry.mode == 2 || entry.mode == 0  { self.mode_2.push(entry.parent.index); }
    }
    pub fn apply(&self, result: &mut AssetTableResult, mode: i32, condition_flags: Option<&AssetTableConditionFlags>) {
        let flags = condition_flags.unwrap_or(AssetTableStaticFields::get().condition_flags);
        if mode == 2 { &self.mode_2 } else { &self.mode_1 }.iter().flat_map(|&i| AssetTable::try_index_get(i))
            .for_each(|entry| { if entry.condition_indexes.test(flags) { result.commit_asset_table(entry); } });
    }
    pub fn is_empty(&self) -> bool { self.mode_1.is_empty() || self.mode_2.is_empty() }
}
pub fn parse_arg_from_name<'a>(line: &'a str, arg_name: &str) -> Option<&'a str> {
    line.split_whitespace().find(|s| s.starts_with(format!("{}=",arg_name).as_str()))
        .and_then(|s| parse_arg_equal(s))
}

pub fn parse_arg_equal(arg: &str) -> Option<&str> { arg.split_once("=").map(|v| v.1) }

pub fn get_job_hashes(arg: &str, is_royal: bool) -> Option<Vec<i32>> {
    let mut hashes = vec![];
    if arg.contains(",") { hashes = arg.split(",").flat_map(|x| JobData::get(x).map(|j| j.parent.hash)).collect::<Vec<_>>(); }
    else  {
        if let Some(h) = JobData::get(arg).map(|j| j.parent.hash) { hashes.push(h); }
        if is_royal {
            if let Some(h) = JobData::get(format!("{}下級", arg)).map(|j| j.parent.hash) { hashes.push(h); }
            if let Some(h) = JobData::get(format!("{}_E", arg)).map(|j| j.parent.hash) { hashes.push(h); }
        }
    }
    if hashes.is_empty() { None } else { Some(hashes) }
}

pub fn parse_label(label: &str) -> (String, i32) {
    if let Some(parsed) = label.split_once(":") { (parsed.0.to_string(), parsed.1.parse::<i32>().unwrap_or(0)) }
    else { (label.to_string(), 0) }
}