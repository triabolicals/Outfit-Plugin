use engage::app::{IStructBase, IStructData_1Methods};
use crate::job_map;
pub fn parse_arg_from_name<'a>(line: &'a str, arg_name: &str) -> Option<&'a str> {
    line.split_whitespace().find(|s| s.starts_with(format!("{}=",arg_name).as_str()))
        .and_then(|s| parse_arg_equal(s))
}

pub fn parse_arg_equal(arg: &str) -> Option<&str> { arg.split_once("=").map(|v| v.1) }

pub fn get_job_hashes(arg: &str, is_royal: bool) -> Option<Vec<i32>> {
    let mut hashes = vec![];
    if arg.contains(",") {
        hashes = arg.split(",").flat_map(|x| job_map(engage::app::JobData::get(format!("JID_{}", x).into()), |j| j.hash()) ).collect::<Vec<_>>(); }
    else  {
        if let Some(h) = job_map(engage::app::JobData::get(format!("JID_{}", arg).into()), |j| j.hash()) { hashes.push(h); }
        if is_royal {
            if let Some(h) = job_map(engage::app::JobData::get(format!("JID_{}下級", arg).into()), |j| j.hash()) { hashes.push(h); }
            if let Some(h) = job_map(engage::app::JobData::get(format!("JID_{}_E", arg).into()), |j| j.hash()) { hashes.push(h); }
        }
    }
    if hashes.is_empty() { None } else { Some(hashes) }
}

pub fn parse_label(label: &str) -> (String, i32) {
    if let Some(parsed) = label.split_once(":") { (parsed.0.to_string(), parsed.1.parse::<i32>().unwrap_or(0)) }
    else { (label.to_string(), 0) }
}