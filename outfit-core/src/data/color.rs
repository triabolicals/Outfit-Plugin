use std::fmt::Display;
use engage::gamedata::assettable::AssetTableResult;
use engage::stream::Stream;
use unity::engine::Color;
use crate::localize::MenuTextCommand;
pub use super::*;

#[derive(Clone, Copy, PartialEq)]
pub struct AssetColor { pub values: [u8; 4], }
impl Default for AssetColor {
    fn default() -> Self { Self { values: [0; 4], } }
}
impl Display for AssetColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}/{}/{}", self.values[0], self.values[1], self.values[2]))
    }
}

impl AssetColor{
    pub(crate) const fn new() -> Self { Self { values: [0; 4] } }
    pub fn set(&mut self, value: u8, index: i32) { if index < 4 { self.values[index as usize] = value; } }
    pub fn set_by_i32(&mut self, value: i32) {
        for x in 0..4 {
            let c = (value >> (x * 8)) as u8;
            self.values[x] = c;
        }
    }
    pub fn has_color(&self) -> bool { self.values[0] != 0 || self.values[1] != 0 || self.values[2] != 0 }
    pub fn is_zero(&self) -> bool { self.values[0] == 0 && self.values[1] == 0 && self.values[2] == 0 }
    pub fn set_result_color(&self, result: &mut AssetTableResult, result_index: usize) {
        if self.has_color() { result.unity_colors[result_index] = self.to_unity_color(); }
    }
    pub fn from_stream(stream: &mut Stream) -> Self {
        let mut values = [0; 4];
        for x in 0..4 { values[x] = stream.read_u8().unwrap_or(0); }
        Self { values }
    }
    pub fn serialize(&self, stream: &mut Stream) -> usize {
        let mut bytes = 0;
        self.values.iter().for_each(|v|{ bytes += stream.write_u8(*v).unwrap(); });
        bytes
    }
    pub fn to_unity_color(&self) -> Color {
        Color {
            r: (self.values[0] as f32) / 255.0,
            g: (self.values[1] as f32) / 255.0,
            b: (self.values[2] as f32) / 255.0,
            a: (self.values[3] as f32) / 255.0
        }
    }
}

#[derive(Default)]
pub struct ColorPreset {
    pub colors: [i32; 8],
    pub engaged: bool,
    pub count: i32,
    pub label: String,
}
impl ColorPreset {
    pub fn from_line(line: &str, is_hair: bool) -> Option<ColorPreset> {
        let mut spilt = line.split_whitespace();
        if let Some(name) = spilt.next() {
            let mut colors: [i32; 8] = [0; 8];
            let mut idx: usize = 0;
            let engaged = name.starts_with("EID_");
            let label = if engaged { name.replace("EID_", "MPID_") } else { name.to_string() };
            while let Some(color) = spilt.next() {
                let color_value= Self::parse_color(color);
                if is_hair && idx < 4 { colors[idx] = color_value; }
                else if idx < 8 {
                    if idx < 4 { colors[idx+4] = color_value; }
                    else { colors[idx-4] = color_value; }
                }
                else { break;}
                idx += 1;
            }
            Some(Self{ colors, engaged,label, count: 0 })
        }
        else { None }
    }
    pub fn parse_color(color: &str) -> i32 {
        color.split(",")
            .flat_map(|x| x.parse::<i32>().ok())
            .enumerate()
            .fold(0, |acc, (i, x)| {
                let value = x << (i * 8);
                acc | value
            })
    }
    pub fn set_color(color: &mut Color, value: i32) {
        color.r = ((value & 255) as f32) / 255.0;
        color.g = (((value >> 8) & 255) as f32) / 255.0;
        color.b = (((value >> 16) & 255) as f32) / 255.0;
    }
    pub fn get_name(&self) -> &'static Il2CppString {
        let s =
            if self.count == 0 { Mess::get(self.label.as_str()) }
            else { format!("{} {}", Mess::get(self.label.as_str()), self.count+1).into() };
        if self.engaged { format!("{} {}", MenuTextCommand::Engage, s).into() } else { s }
    }
}