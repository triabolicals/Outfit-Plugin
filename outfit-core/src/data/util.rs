use engage::app::{IAssetTable, IAssetTableMethods, IAssetTable_ConditionIndexesMethods, IAssetTable_ResultMethods, IStructBase, IStructData_1Methods};
use unity::Cast;
use crate::job_map;

#[derive(Default)]
pub struct AssetTableIndexes {
    pub mode_1: Vec<i32>,
    pub mode_2: Vec<i32>,
}
impl AssetTableIndexes {
    pub fn add(&mut self, entry: engage::app::AssetTable) {
        let mode = entry.get_mode().value;
        if mode == 1 || mode == 0 { self.mode_1.push(entry.index()); }
        if mode == 2 || mode == 0  { self.mode_2.push(entry.index()); }
    }
    pub fn apply(&self, result: engage::app::AssetTable_Result, mode: i32, condition_flags: engage::app::AssetTable_ConditionFlags) {
        if condition_flags.is_null() { return; }
        if mode == 2 { &self.mode_2 } else { &self.mode_1 }.iter()
            .map(|&i| engage::app::AssetTable::try_get_2(i))
            .for_each(|entry| { if entry.m_condition_indexes().test(condition_flags) { result.commit_4(entry); } });
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
    if arg.contains(",") { hashes = arg.split(",").flat_map(|x| job_map(engage::app::JobData::get(arg.into()), |j| j.hash()) ).collect::<Vec<_>>(); }
    else  {
        if let Some(h) = job_map(engage::app::JobData::get(arg.into()), |j| j.hash()) { hashes.push(h); }
        if is_royal {
            if let Some(h) = job_map(engage::app::JobData::get(format!("{}下級", arg).into()), |j| j.hash()) { hashes.push(h); }
            if let Some(h) = job_map(engage::app::JobData::get(format!("{}_E", arg).into()), |j| j.hash()) { hashes.push(h); }
        }
    }
    if hashes.is_empty() { None } else { Some(hashes) }
}

pub fn parse_label(label: &str) -> (String, i32) {
    if let Some(parsed) = label.split_once(":") { (parsed.0.to_string(), parsed.1.parse::<i32>().unwrap_or(0)) }
    else { (label.to_string(), 0) }
}