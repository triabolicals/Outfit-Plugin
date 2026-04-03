use engage::gameicon::*;
use unity::engine::Sprite;
use unity::prelude::Il2CppClassData;

#[repr(u8)]
#[derive(PartialEq, Copy, Clone)]
pub enum CustomMenuIcon {
    Clothes,
    Body,
    Head,
    Hair,
    Talk,
    AccFace,
    Shield,
    StarBlank,
    Star,
    EngageCommon,
    Engaged(u8),
    Rare,
    KeyItem,
    TimeCrystal,
    RedEngWep,
    GreenEngWep,
    BlueEngWep,
    WhiteEngWep,
    Mount(u8),
    Armor,
    Gift,
    Horse,
    Dragon,
    Fly,
    Weapon,
    SolaTail,
    Satchel,
    GiftCategory,
    UnitAccessory{kind: u8},
    TalkStory,
    Day,
    SilverCard,
    NoIcon,

}
impl CustomMenuIcon {
    pub fn get_system_label(&self) -> Option<&'static str> {
        match self {
            Self::Clothes => Some("Clothes"),
            Self::Body => Some("Body"),
            Self::Head => Some("SolaHeadCategory"),
            Self::Hair => Some("SolaHead"),
            Self::Talk => Some("TalkRelianceOutline"),
            Self::AccFace => Some("Face"),
            Self::KeyItem => Some("KeyItem"),
            Self::TimeCrystal => Some("TimeCrystal"),
            Self::EngageCommon => Some("EngCommon"),
            Self::Gift => Some("Gift"),
            Self::GiftCategory => Some("GiftCategory"),
            Self::Rare => Some("Rare"),
            Self::StarBlank => Some("StarBlank_Small"),
            Self::Star => Some("Star_Small"),
            Self::SolaTail => Some("SolaTail"),
            Self::RedEngWep => Some("EngWepAtk"),
            Self::GreenEngWep => Some("EngWepSpd"),
            Self::BlueEngWep => Some("EngWepDef"),
            Self::WhiteEngWep => Some("EngWepSkill"),
            Self::Satchel => Some("EnchantSeal"),
            Self::TalkStory => Some("TalkStoryOutline"),
            Self::Weapon => Some("Weapon"),
            Self::Day => Some("Day"),
            Self::SilverCard => Some("SilverCard"),
            _ => None,
        }
    }
    pub fn get_icon(&self) -> Option<&'static mut Sprite> {
        if let Some(system_label) = self.get_system_label() { GameIcon::try_get_system(system_label) }
        else {
            match self {
                Self::Engaged(i) => { GameIcon::class().get_static_fields::<GameIconStaticFields>().god_symbol.try_get(crate::EMBLEM[*i as usize].0) }
                Self::UnitAccessory {kind} => { GameIcon::try_get_accessory_kind(*kind as i32) }
                Self::Shield => { GameIcon::try_get_item("Byleth_AegisShield") }
                Self::Mount(i) => {
                    let icon_label = match i { 2 => "Dragon", 3|4 => "Fly", _ => "Horse", };
                    GameIcon::try_get_efficacy(icon_label, true)
                }
                Self::Armor => { GameIcon::try_get_efficacy("Armor", true) }
                Self::Horse => { GameIcon::try_get_efficacy("Horse", true) }
                Self::Dragon => { GameIcon::try_get_efficacy("Dragon", true) }
                Self::Fly => { GameIcon::try_get_efficacy("Fly", true) }
                _ => None,
            }
        }
    }
}