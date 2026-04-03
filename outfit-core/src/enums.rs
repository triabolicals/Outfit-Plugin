use std::fmt::Display;
use engage::unit::Gender;

pub const BODY_EXPRESS: [&str; 47] = [
    "EmoNormal", "EmoAngry", "EmoDeny", "EmoDeny2", "EmoGive", "EmoJoy", "EmoJoy2", "EmoLookR", "EmoLookL",
    "EmoPray","EmoPray2","EmoProud","EmoRelax","EmoSad","EmoSad2","EmoShy","EmoSurprise","EmoSurprise2","EmoTalk",
    "EmoTalk2","EmoThink","EmoTired","EmoWave","EmoPosing","EmoPosing2","EmoCheer1","EmoCheer2","EmoFishShow","EmoScatterR",
    "EmoScatterL","EmoSing","EmoGuitar","EmoStrokeStand","EmoWateringR","EmoWateringL","EmoPick1","EmoPick2","EmoPick3",
    "ReactionGood","ReactionBad","Battle_Sword","Battle_Lance","Battle_Bow","Battle_Axe","Battle_Knife","Battle_Fist",
    "Battle_Magic",
];
pub const FACIAL_STATES: [&str; 18] = [
    "Normal", "Angry", "Surprise", "Relax", "Pain", "Die", "Smile", "Strike", "Serious", "Sad", "Shy", "StandBy", "Status",
    "LipA", "LipI", "LipO", "LipE", "LipU"
];
pub const EMBLEM: [(&str, &str); 24] = [
    ("Marth", "Mar"), ("Siglud", "Sig"), ("Celica", "Cel"), ("Micaiah", "Mic"),
    ("Roy",  "Roy"), ("Leaf", "Lei"), ("Lucina", "Luc"), ("Lin", "Lyn"),
    ("Ike", "Ike"), ("Byleth", "Byl"), ("Kamui", "Cor"), ("Eirik", "Eir"),
    ("Edelgard", "Thr"), ("Tiki", "Tik"), ("Hector",   "Hec"),
    ("Veronica",  "Ver"), ("Senerio",   "Sor"), ("Camilla",  "Cmi"), ("Chrom",    "Chr"),
    ("Lueur",    "Ler"), ("Dimitri",  "Dim"), ("Claude",   "Cla"), ("Reflet", "Rbi"), ("Ephraim", "Eph")
];
pub const SCALE: [&str; 19] = [
    "ScaleAll", "ScaleHead", "ScaleNeck", "ScaleTorso", "ScaleShoulders", "ScaleArms", "ScaleHands",
    "ScaleLegs", "ScaleFeet", "VolumeArms", "VolumeLegs", "VolumeBust", "VolumeAbdomen", "VolumeTorso",
    "VolumeScaleArms", "VolumeScaleLegs", "MapScaleAll", "MapScaleHead", "MapScaleWing"
];
pub const V_EVENTS: &[&str] = &["V_Title_01", "V_Arena_Name", "V_Wideuse_13", "V_Engage_Respond", "V_Title_02"];
pub const COLOR_MASK: [&str; 8] = ["MaskColor100", "MaskColor075", "MaskColor050", "MaskColor025", "Hair", "Grad", "Skin", "Toon"];

#[derive(Copy, Clone, PartialEq, Hash, Eq)]
pub enum Mount {
    None,
    Cav,
    Wolf,
    Wyvern,
    Pegasus,
    Griffin,
}

impl From<i32> for Mount {
    fn from(value: i32) -> Self {
        match value {
            1 => Mount::Cav,    //BX
            2 => Mount::Wolf,   //CX
            3 => Mount::Wyvern, //DX
            4 => Mount::Pegasus,    //EF
            5 => Mount::Griffin,    //FX
            _ => Mount::None,   //AX
        }
    }
}
impl From<Mount> for i32 {
    fn from(value: Mount) -> Self {
        match value {
            Mount::None => { 0 }
            Mount::Cav => { 1 }
            Mount::Wolf => { 2 }
            Mount::Wyvern => { 3 }
            Mount::Pegasus => { 4 }
            Mount::Griffin => { 5 }
        }
    }
}
impl Display for Mount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mount::Cav => write!(f, "Cav"),
            Mount::Wolf => write!(f, "Wolf"),
            Mount::Wyvern => write!(f, "Wyvern"),
            Mount::Pegasus => write!(f, "Pegasus"),
            Mount::Griffin => write!(f, "Griffin"),
            Mount::None => write!(f, "Infantry"),
        }
    }
}
impl Mount {
    const MOUNTSEARCH: [[&'static str; 6]; 5] = [
        ["uRig_Hors", "BM", "BF", "B*", "BR", "B*"],
        ["uRig_Wolf", "CM", "CF", "CR", "CT", "C*"],
        ["uRig_Drag", "DF", "DM", "DR", "DT", "D*"],
        ["uRig_Pega", "EF", "ER", "Slp", "Wng0", "Wng0"],
        ["uRig_Grif", "FF", "FM", "FR", "F*", "Wng1F"],
    ];
    pub fn from(str: impl AsRef<str>) -> Mount {
        let str = str.as_ref();
        Self::MOUNTSEARCH.iter().position(|x| x.iter().any(|s| str.contains(*s)))
            .map(|pos| Self::from_i32(pos as i32 + 1))
            .unwrap_or(Mount::None)
    }
    pub fn from_i32(value: i32) -> Mount {
        match value {
            1 => Mount::Cav,    //BX
            2 => Mount::Wolf,   //CX
            3 => Mount::Wyvern, //DX
            4 => Mount::Pegasus,    //EF
            5 => Mount::Griffin,    //FX
            _ => Mount::None,   //AX
        }
    }
    pub fn get_ride_race(&self) -> &'static str {
        match self {
            Mount::None => "",
            Mount::Cav => "BR",
            Mount::Wyvern => "DR",
            Mount::Wolf => "CR",
            Mount::Pegasus => "ER",
            Mount::Griffin => "FR",
        }
    }
    pub fn get_default_asset(&self, rig: bool) -> &'static str {
        if rig {
            match self {
                Mount::None => "null",
                Mount::Cav => "uRig_HorsR",
                Mount::Wolf => "uRig_WolfR",
                Mount::Wyvern => "uRig_DragR",
                Mount::Pegasus => "uRig_PegaR",
                Mount::Griffin => "uRig_GrifR",
            }
        }
        else {
            match self {
                Mount::None => "null",
                Mount::Cav => "Cav0B",
                Mount::Wolf => "Cav2C",
                Mount::Wyvern => "Wng2D",
                Mount::Pegasus => "Wng0E",
                Mount::Griffin => "Wng1F",
            }
        }.into()
    }
    pub fn get_gender_race(&self, gender: Gender) -> &'static str {
        let female = Gender::Female == gender;
        if gender == Gender::Other {
            "AT"
        }
        else {
            match self {
                Mount::None => if female { "AF" } else { "AM" },
                Mount::Cav => if female { "BF" } else { "BM" },
                Mount::Wolf => if female { "CF" } else { "CM" },
                Mount::Wyvern => if female { "DF" } else { "DM" },
                Mount::Pegasus => "EF",
                Mount::Griffin => if female { "FF" } else { "FM" },
            }
        }

    }
    pub fn determine_gender(str: impl AsRef<str>) -> Option<(Self, Gender)> {
        ["AM", "BM", "CM", "DM", "EM","FM"].iter()
            .position(|x| str.as_ref().contains(x))
            .map(|x|{ (Mount::from_i32(x as i32), Gender::Male) })
            .or_else(||
                ["AF", "BF", "CF", "DF", "EF","FF"].iter()
                    .position(|x| str.as_ref().contains(x))
                    .map(|x| (Mount::from_i32(x as i32), Gender::Female))
            )
    }

}